use core::mem::ManuallyDrop;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::font::{FontBufferColoredGlyph, FontBufferGlyph, FontChar, FontRenderer};
use crate::helper::{likely, unlikely};

#[derive(Debug, Clone)]
pub enum RenderResult {
    /// 空 (完全透明，什么都不绘制)
    /// - 仅在缓存返回时使用，正常渲染不应该出现此结果
    ///
    /// Nothing is drawn. (Transparent)
    /// - This variant is only used when returning from the cache, and normal rendering should not produce this result.
    Empty,
    /// 背景 (完全被背景色覆盖)
    ///
    /// Completely covered by the background color.
    Blank,
    /// 普通的次像素字形
    Subpix(Arc<FontBufferGlyph>),
    /// 有颜色的字形 (如 Emoji)
    Colored(Arc<FontBufferColoredGlyph>),
}

impl PartialEq for RenderResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RenderResult::Empty, RenderResult::Empty) => true,
            (RenderResult::Blank, RenderResult::Blank) => true,
            (RenderResult::Subpix(a), RenderResult::Subpix(b)) => Arc::ptr_eq(a, b),
            (RenderResult::Colored(a), RenderResult::Colored(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for RenderResult {}

#[derive(Debug, Clone)]
pub enum RenderResultRef<'r> {
    Empty,
    Blank,
    Subpix(&'r Arc<FontBufferGlyph>),
    Colored(&'r Arc<FontBufferColoredGlyph>),
}

impl PartialEq for RenderResultRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RenderResultRef::Empty, RenderResultRef::Empty) => true,
            (RenderResultRef::Blank, RenderResultRef::Blank) => true,
            (RenderResultRef::Subpix(a), RenderResultRef::Subpix(b)) => Arc::ptr_eq(a, b),
            (RenderResultRef::Colored(a), RenderResultRef::Colored(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for RenderResultRef<'_> {}

impl RenderResult {
    pub fn split(self) -> (RenderResultType, RenderResultUnion) {
        match self {
            RenderResult::Empty => (RenderResultType::Empty, RenderResultUnion::empty()),
            RenderResult::Blank => (RenderResultType::Blank, RenderResultUnion::blank()),
            RenderResult::Subpix(glyph) => {
                (RenderResultType::Subpix, RenderResultUnion::glyph(ManuallyDrop::new(glyph)))
            }
            RenderResult::Colored(glyph) => {
                (RenderResultType::Colored, RenderResultUnion::colored(ManuallyDrop::new(glyph)))
            }
        }
    }

    pub fn as_ref(&self) -> RenderResultRef<'_> {
        match self {
            RenderResult::Empty => RenderResultRef::Empty,
            RenderResult::Blank => RenderResultRef::Blank,
            RenderResult::Subpix(glyph) => RenderResultRef::Subpix(glyph),
            RenderResult::Colored(glyph) => RenderResultRef::Colored(glyph),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderResultType {
    Empty,
    Blank,
    Subpix,
    Colored,
}

#[repr(align(8))]
pub union RenderResultUnion {
    empty: (),
    blank: (),
    subpix: ManuallyDrop<Arc<FontBufferGlyph>>,
    colored: ManuallyDrop<Arc<FontBufferColoredGlyph>>,
}

impl RenderResultType {
    #[expect(unsafe_code)]
    pub unsafe fn merge(self, data: RenderResultUnion) -> RenderResult {
        match self {
            RenderResultType::Empty => RenderResult::Empty,
            RenderResultType::Blank => RenderResult::Blank,
            RenderResultType::Subpix => RenderResult::Subpix(unsafe { ManuallyDrop::into_inner(data.subpix) }),
            RenderResultType::Colored => RenderResult::Colored(unsafe { ManuallyDrop::into_inner(data.colored) }),
        }
    }
}

#[expect(unsafe_code)]
impl RenderResultUnion {
    pub const fn empty() -> Self {
        Self { empty: () }
    }

    pub const fn blank() -> Self {
        Self { blank: () }
    }

    pub const fn glyph(subpix: ManuallyDrop<Arc<FontBufferGlyph>>) -> Self {
        Self { subpix }
    }

    pub const fn colored(colored: ManuallyDrop<Arc<FontBufferColoredGlyph>>) -> Self {
        Self { colored }
    }

    pub unsafe fn clone(&self, ty: &RenderResultType) -> Self {
        match ty {
            RenderResultType::Empty => Self::empty(),
            RenderResultType::Blank => Self::blank(),
            RenderResultType::Subpix => Self::glyph(unsafe { self.subpix.clone() }),
            RenderResultType::Colored => Self::colored(unsafe { self.colored.clone() }),
        }
    }

    pub unsafe fn as_ref(&self, ty: &RenderResultType) -> RenderResultRef<'_> {
        match ty {
            RenderResultType::Empty => RenderResultRef::Empty,
            RenderResultType::Blank => RenderResultRef::Blank,
            RenderResultType::Subpix => RenderResultRef::Subpix(unsafe { &self.subpix }),
            RenderResultType::Colored => RenderResultRef::Colored(unsafe { &self.colored }),
        }
    }
}

// ===== ===== ===== ===== ===== AAAAA ===== ===== ===== ===== ===== //

pub struct FontBuffer {
    /// 字形渲染器
    renderers: Vec<Box<dyn FontRenderer>>,
    /// 已缓存的字形 (未上色)
    glyphs: BTreeMap<FontChar, Arc<FontBufferGlyph>>,
    /// 字体宽度
    pub width: u32,
    /// 字体高度
    pub height: u32,
}

impl Default for FontBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl FontBuffer {
    pub fn new() -> Self {
        Self {
            renderers: Vec::new(),
            glyphs: BTreeMap::new(),
            width: 0,
            height: 0,
        }
    }
}

impl FontBuffer {
    /// 渲染一个字符 (当缓存未命中时)
    fn render(&mut self, ch: FontChar) -> RenderResult {
        for renderer in &mut self.renderers {
            let glyph = renderer.get(ch.ch, ch.bold, ch.italic);
            let glyph = glyph.align_to(self.width * ch.width as u32, self.height);
            let glyph = Arc::new(FontBufferGlyph::from(glyph));
            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }
            self.glyphs.insert(ch, glyph.clone());
            return RenderResult::Subpix(glyph);
        }
        let glyph = Arc::new(FontBufferGlyph::blank(self.width, self.height));
        self.glyphs.insert(ch, glyph.clone());
        RenderResult::Subpix(glyph)
    }
}

impl FontBuffer {
    pub fn add_renderer(&mut self, renderer: Box<dyn FontRenderer>) {
        if self.renderers.is_empty() {
            self.width = renderer.size().0 as u32;
            self.height = renderer.size().1 as u32;
        }
        self.renderers.push(renderer);
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
    }

    /// 每次终端刷新一帧都要在字体缓存中调用一次 tick 方法
    ///
    /// 用于动态调整缓存策略
    pub fn tick(&mut self) {
        // 已删除
    }

    pub fn sizeof(&self, ch: FontChar) -> (usize, usize) {
        if let Some(glyph) = self.glyphs.get(&ch) {
            return (glyph.width as usize, glyph.height as usize);
        }
        for renderer in &self.renderers {
            if let Some((width, height)) = renderer.sizeof(ch.ch, ch.bold, ch.italic) {
                return (width, height);
            }
        }
        (0, 0)
    }

    pub fn get(&mut self, ch: FontChar) -> RenderResult {
        if unlikely(ch.ch == '\0' || self.renderers.is_empty()) {
            return RenderResult::Blank;
        }
        let glyph = self.glyphs.get(&ch);
        if likely(glyph.is_some()) {
            return RenderResult::Subpix(glyph.unwrap().clone());
        }
        self.render(ch)
    }

    /// 尝试释放字形缓存，所有字形释放前都必须调用此方法
    ///
    /// Tries to release the glyph cache.
    /// This method must be called before releasing any glyph.
    pub fn unref(&mut self, _ch: FontChar, buf: RenderResult) {
        match buf {
            RenderResult::Empty => {}
            RenderResult::Blank => {}
            RenderResult::Subpix(_) => {
                // We believe that storing unused uncolored glyphs will not take up too much memory.
            }
            RenderResult::Colored(_) => {
                // We believe that storing unused colored glyphs will not take up too much memory.
            }
        }
    }
}
