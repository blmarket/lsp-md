use std::borrow::Cow;
use std::ops::{Range, RangeBounds};
use std::slice::SliceIndex;

pub trait SliceAccess {
    fn slice<'a, R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        &'a self,
        r: R,
    ) -> Cow<'a, str>;
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub(super) title: Range<usize>,
    pub(super) range: Range<usize>,
}

pub trait BasicDocument {
    type Output: AsRef<[Section]>;

    fn sections(&self) -> Self::Output;
}

pub trait DocumentExt<'a> {
    type Output: Into<Cow<'a, str>>;

    fn title(&'a self, index: usize) -> anyhow::Result<Self::Output>;
    fn text(&'a self, index: usize) -> anyhow::Result<Self::Output>;
}

impl<'a, T: BasicDocument + SliceAccess> DocumentExt<'a> for T {
    type Output = Cow<'a, str>;

    fn title(&'a self, index: usize) -> anyhow::Result<Self::Output> {
        Ok(self.slice(self.sections().as_ref()[index].title.clone()))
    }

    fn text(&'a self, index: usize) -> anyhow::Result<Self::Output> {
        Ok(self.slice(self.sections().as_ref()[index].range.clone()))
    }
}
