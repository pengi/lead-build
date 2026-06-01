use std::{collections::BTreeMap, fmt::Display};

#[derive(Debug)]
pub enum Error {
    DupKey(String),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::DupKey(key) => format!("Duplicate key: {}", key),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub struct ImMap<T: Display + Clone + PartialEq>(BTreeMap<String, T>);

impl<T> Display for ImMap<T>
where
    T: Clone + PartialEq + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{ ")?;
        for (key, value) in self.0.iter() {
            f.write_str(key)?;
            f.write_str(" = ")?;
            value.fmt(f)?;
            f.write_str("; ")?;
        }
        f.write_str("}")?;
        Ok(())
    }
}

impl<T> Default for ImMap<T>
where
    T: Display + Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ImMap<T>
where
    T: Display + Clone + PartialEq,
{
    pub fn new() -> ImMap<T> {
        Self(BTreeMap::new())
    }

    pub fn single(key: String, value: T) -> ImMap<T> {
        Self::new().set(key, value).unwrap()
    }

    pub fn from(fields: impl Iterator<Item = (String, T)>) -> Result<ImMap<T>> {
        let mut ret: ImMap<T> = Default::default();
        for (key, value) in fields {
            ret = ret.set(key, value)?
        }
        Ok(ret)
    }

    pub fn merge(self, other: &ImMap<T>) -> ImMap<T> {
        let mut out = self.0;
        let mut to_update = other.0.clone();
        out.append(&mut to_update);
        ImMap(out)
    }

    pub fn set(self, key: String, value: T) -> Result<ImMap<T>> {
        let mut map = self;
        let res = map.0.insert(key.clone(), value);
        match res {
            Some(_) => Err(Error::DupKey(key)),
            None => Ok(map),
        }
    }

    pub fn unset(self, key: &str) -> ImMap<T> {
        let mut map = self;
        map.0.remove(key);
        map
    }

    pub fn get(&self, key: &str) -> Option<T> {
        self.0.get(key).cloned()
    }

    pub fn map<B, F>(&self, f: F) -> ImMap<B>
    where
        F: Fn(&T) -> B,
        B: Display + Clone + PartialEq,
    {
        ImMap::from(self.0.iter().map(|(name, value)| (name.clone(), f(value)))).unwrap()
    }
}
