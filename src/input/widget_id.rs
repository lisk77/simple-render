use std::{fmt, sync::Arc};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(Arc<str>);

impl WidgetId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<&'static str> for WidgetId {
    fn from(id: &'static str) -> Self {
        Self::new(id)
    }
}

impl From<String> for WidgetId {
    fn from(id: String) -> Self {
        Self::new(Arc::<str>::from(id))
    }
}

impl From<Arc<str>> for WidgetId {
    fn from(id: Arc<str>) -> Self {
        Self::new(id)
    }
}

impl From<u64> for WidgetId {
    fn from(id: u64) -> Self {
        Self::from(id.to_string())
    }
}
