use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Clone, PartialEq, Debug)]
pub struct VirtPath {
    root: String,
    locked_parts: Vec<String>,
    parts: Vec<String>,
}

impl Display for VirtPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.root)?;
        for part in self.locked_parts.iter() {
            write!(f, "/{}", part)?;
        }
        write!(f, "/$")?;
        for part in self.parts.iter() {
            write!(f, "/{}", part)?;
        }
        Ok(())
    }
}

impl VirtPath {
    pub fn new(root: impl ToString) -> VirtPath {
        VirtPath {
            root: root.to_string(),
            locked_parts: vec![],
            parts: vec![],
        }
    }

    pub fn lock(self) -> VirtPath {
        let mut res = self;
        res.locked_parts.append(&mut res.parts);
        res
    }

    pub fn parent(self) -> Option<VirtPath> {
        let mut out = self;
        out.parts.pop().map(|_| out)
    }

    pub fn step(self, elem: impl ToString) -> Option<VirtPath> {
        let mut out: VirtPath = self;
        let elem = elem.to_string();
        match elem.as_str() {
            ".." => out.parent(),
            "." => Some(out),
            _ => {
                out.parts.push(elem);
                Some(out)
            }
        }
    }

    pub fn to_path_buf(&self, path_refs: &BTreeMap<String, PathBuf>) -> Option<PathBuf> {
        let root_path = path_refs.get(&self.root)?;
        let mut cur_path = root_path.clone();
        for part in self.locked_parts.iter() {
            cur_path.push(part);
        }
        for part in self.parts.iter() {
            cur_path.push(part);
        }
        Some(cur_path)
    }

    pub fn virtualize(
        path: &Path,
        root: impl ToString,
        path_refs: &mut BTreeMap<String, PathBuf>,
    ) -> Option<VirtPath> {
        if path_refs
            .insert(root.to_string(), path.parent().unwrap().to_path_buf())
            .is_some()
        {
            return None;
        }
        Some(VirtPath {
            root: root.to_string(),
            locked_parts: vec![],
            parts: vec![path.file_name().unwrap().to_str().unwrap().into()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_down() {
        let path = VirtPath::new("root");
        assert_eq!(path.to_string().as_str(), "[root]/$");
        let path = path.step("test").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/$/test");
        let path = path.step("b").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/$/test/b");
        let path = path.step("..").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/$/test");
        let path = path.step(".").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/$/test");
        let path = path.step("..").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/$");
        assert!(path.step("..").is_none());
    }

    #[test]
    fn test_lock() {
        let path = VirtPath::new("root");
        assert_eq!(path.to_string().as_str(), "[root]/$");
        let path = path.step("test").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/$/test");
        let path = path.lock();
        assert_eq!(path.to_string().as_str(), "[root]/test/$");
        assert!(path.step("..").is_none());
    }

    #[test]
    fn test_relock() {
        let path = VirtPath::new("root");
        assert_eq!(path.to_string().as_str(), "[root]/$");
        let path = path.step("test").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/$/test");
        let path = path.lock();
        assert_eq!(path.to_string().as_str(), "[root]/test/$");
        let path = path.step("b").unwrap();
        assert_eq!(path.to_string().as_str(), "[root]/test/$/b");
        let path = path.lock();
        assert_eq!(path.to_string().as_str(), "[root]/test/b/$");
    }

    #[test]
    fn test_path_root_eq() {
        let path_a1 = VirtPath::new("a");
        let path_a2 = VirtPath::new("a");
        let path_b = VirtPath::new("b");
        assert_eq!(path_a1, path_a2);
        assert_ne!(path_a1, path_b);
        assert_ne!(path_a2, path_b);
    }

    #[test]
    fn test_path_path_eq() {
        let base = VirtPath::new("a");
        let cur = VirtPath::new("a");
        let cur = cur.step("test").unwrap();
        assert_ne!(base, cur);
        let cur = cur.step("..").unwrap();
        assert_eq!(base, cur);
    }

    #[test]
    fn test_virtpath_to_path_buf() {
        let mut refs = BTreeMap::new();
        refs.insert(String::from("a"), PathBuf::from("./test_a"));
        refs.insert(String::from("b"), PathBuf::from("./test_b"));

        assert_eq!(
            VirtPath::new("a").to_path_buf(&refs).unwrap(),
            PathBuf::from("./test_a")
        );

        assert_eq!(
            VirtPath::new("a")
                .step("hej")
                .unwrap()
                .to_path_buf(&refs)
                .unwrap(),
            PathBuf::from("./test_a/hej")
        );

        assert_eq!(
            VirtPath::new("a")
                .step("hej")
                .unwrap()
                .lock()
                .to_path_buf(&refs)
                .unwrap(),
            PathBuf::from("./test_a/hej")
        );

        assert_eq!(
            VirtPath::new("b")
                .step("hej")
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf(&refs)
                .unwrap(),
            PathBuf::from("./test_b")
        );
    }

    #[test]
    fn test_from_path() {
        let mut refs = BTreeMap::new();
        let filepath = PathBuf::from("./test/file.txt");
        let virtpath = VirtPath::virtualize(&filepath, "root", &mut refs).unwrap();

        assert_eq!(
            virtpath,
            VirtPath {
                root: "root".into(),
                locked_parts: vec![],
                parts: vec!["file.txt".into()]
            }
        );
        assert_eq!(
            refs,
            BTreeMap::from([(String::from("root"), PathBuf::from("./test"))])
        )
    }
}
