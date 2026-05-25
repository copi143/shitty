import java.io.BufferedReader
import java.io.InputStreamReader
import java.lang.foreign.Arena
import java.lang.foreign.FunctionDescriptor
import java.lang.foreign.Linker
import java.lang.foreign.MemorySegment
import java.lang.foreign.SymbolLookup
import java.lang.foreign.ValueLayout
import java.lang.invoke.MethodHandle
import java.nio.file.Path
import kotlin.io.path.Path

private const val DEFAULT_COLS = 80
private const val DEFAULT_ROWS = 24
private const val SCREEN_BUFFER_BYTES = 128 * 1024

private data class NativeApi(
    val terminalNew: MethodHandle,
    val terminalFree: MethodHandle,
    val terminalResize: MethodHandle,
    val terminalFeedUtf8: MethodHandle,
    val terminalSnapshotUtf8: MethodHandle,
)

fun main() {
    val libraryPath = locateLibrary()
    val linker = Linker.nativeLinker()

    Arena.ofConfined().use { arena ->
        val lookup = SymbolLookup.libraryLookup(libraryPath, arena)
        val api = loadApi(linker, lookup)

        val terminal = api.terminalNew.invokeExact(DEFAULT_COLS, DEFAULT_ROWS) as MemorySegment
        try {
            val stdin = BufferedReader(InputStreamReader(System.`in`))

            println("Kotlin FFM terminal demo")
            println("Library: $libraryPath")
            println("输入文本后回车；输入 :quit 退出。")
            println()

            renderScreen(api, terminal, arena)

            while (true) {
                print("> ")
                System.out.flush()

                val line = stdin.readLine() ?: break
                if (line == ":quit") break

                val input = arena.allocateFrom(line + "\n")
                api.terminalFeedUtf8.invokeExact(
                    terminal,
                    input,
                    line.encodeToByteArray().size.toLong() + 1L
                )

                renderScreen(api, terminal, arena)
            }
        } finally {
            api.terminalFree.invokeExact(terminal)
        }
    }
}

private fun renderScreen(api: NativeApi, terminal: MemorySegment, arena: Arena) {
    val buffer = arena.allocate(SCREEN_BUFFER_BYTES.toLong())
    val written = api.terminalSnapshotUtf8.invokeExact(
        terminal,
        buffer,
        SCREEN_BUFFER_BYTES.toLong()
    ) as Long

    if (written <= 0L) {
        return
    }

    print("\u001b[H\u001b[2J")
    println(buffer.getUtf8String(0))
    System.out.flush()
}

private fun loadApi(linker: Linker, lookup: SymbolLookup): NativeApi {
    return NativeApi(
        terminalNew = linker.downcallHandle(
            lookup.find("shitty_terminal_new").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.JAVA_INT, ValueLayout.JAVA_INT)
        ),
        terminalFree = linker.downcallHandle(
            lookup.find("shitty_terminal_free").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        ),
        terminalResize = linker.downcallHandle(
            lookup.find("shitty_terminal_resize").orElseThrow(),
            FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.JAVA_INT, ValueLayout.JAVA_INT)
        ),
        terminalFeedUtf8 = linker.downcallHandle(
            lookup.find("shitty_terminal_feed_utf8").orElseThrow(),
            FunctionDescriptor.of(
                ValueLayout.JAVA_LONG,
                ValueLayout.ADDRESS,
                ValueLayout.ADDRESS,
                ValueLayout.JAVA_LONG
            )
        ),
        terminalSnapshotUtf8 = linker.downcallHandle(
            lookup.find("shitty_terminal_snapshot_utf8").orElseThrow(),
            FunctionDescriptor.of(
                ValueLayout.JAVA_LONG,
                ValueLayout.ADDRESS,
                ValueLayout.ADDRESS,
                ValueLayout.JAVA_LONG
            )
        ),
    )
}

private fun locateLibrary(): Path {
    val envPath = System.getenv("SHITTY_LIB_PATH")?.takeIf { it.isNotBlank() }
    if (envPath != null) {
        return Path.of(envPath).toAbsolutePath().normalize()
    }

    val osName = System.getProperty("os.name").lowercase()
    val fileName = when {
        osName.contains("linux") -> "libshitty.so"
        osName.contains("mac") -> "libshitty.dylib"
        osName.contains("win") -> "shitty.dll"
        else -> "libshitty.so"
    }

    return Path.of(System.getProperty("user.dir"))
        .resolve("../../target/debug/$fileName")
        .toAbsolutePath()
        .normalize()
}
