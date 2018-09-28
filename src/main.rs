use std::env;
use std::fs::{self, Metadata};
use std::io::{self, Error, ErrorKind};
use std::path::PathBuf;
use std::borrow::Cow;

fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in &args[1..] {
        tree(PathBuf::from(arg));
    }
}

fn tree(root: PathBuf) {
    let pathy = MyFuckingPath::new(root).unwrap();
    MyFuckingPrinter::new().rustree(pathy).unwrap();
}

#[derive(Clone)]
enum Bar {
    I,
    T,
    L,
    X,
}

impl Bar {
    fn str(&self) -> &str {
        use Bar::*;
        match *self {
            I => "│   ",
            L => "└── ",
            T => "├── ",
            X => "    ",
        }
    }
}

struct MyFuckingPrinter {
    bar: Vec<bool>,
    is_last: bool,
}

impl MyFuckingPrinter {
    fn new() -> Self {
        MyFuckingPrinter {
            bar: vec![],
            is_last: false,
        }
    }

    fn rustree(&mut self, path: MyFuckingPath) -> io::Result<()> {
        self.print_tree_bars();
        println!("{}", path.summary()?);

        if let Folder = path.file_type {
            self.bar.push(true);
            let mut children: Vec<MyFuckingPath> = path.children()?.collect::<io::Result<_>>()?;
            children.sort();
            let num_children = children.len();
            for (i, child) in children.drain(..).enumerate() {
                self.is_last = i + 1 == num_children;
                self.rustree(child)?;
            }
            self.bar.pop();
        }

        Ok(())
    }

    fn print_tree_bars(&mut self) {
        let mut s = String::from("");
        for i in 0..self.bar.len() {
            let is_last = i == self.bar.len() - 1;
            let barred = self.bar[i];
            s.push_str(match (barred, is_last, self.is_last) {
                (false, _, _) => Bar::X,
                (true, false, _) => Bar::I,
                (true, true, false) => Bar::T,
                (true, true, true) => {
                    self.bar[i] = false;
                    Bar::L
                }
            }.str())
        }
        print!("{}", s);
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum FileType {
    Folder,
    File,
    SymLink,
}
use FileType::*;

impl FileType {
    fn new(metadata: Metadata) -> Option<Self> {
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            Some(Folder)
        } else if file_type.is_file() {
            Some(File)
        } else if file_type.is_symlink() {
            Some(SymLink)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct MyFuckingPath {
    file_type: FileType,
    path: PathBuf,
}

impl MyFuckingPath {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;
        let err_body = Error::new(ErrorKind::Other, "WTF is this?");
        let file_type = FileType::new(metadata).ok_or(err_body)?;

        Ok(MyFuckingPath {
            path: path,
            file_type: file_type,
        })
    }

    fn printable_name(&self) -> Cow<str> {
        let os_str = self.path.file_name().unwrap_or_else(|| self.path.as_os_str());
        os_str.to_string_lossy()
    }

    pub fn summary(&self) -> io::Result<String> {
        fn end_with_slash(s: &str) -> String {
            if s.ends_with("/") {
                String::from(s)
            } else {
                format!("{}/", s)
            }
        }

        let printable_name = self.printable_name();
        Ok(match self.file_type {
            File => String::from(printable_name),
            Folder => end_with_slash(&printable_name),
            SymLink => {
                let target = self.path.read_link()?;
                format!("{} -> {}", printable_name, target.to_string_lossy())
            }
        })
    }

    pub fn children(&self) -> io::Result<MyFuckingChildren> {
        self.path.read_dir().map(MyFuckingChildren::new)
    }
}

struct MyFuckingChildren {
    read_dir: fs::ReadDir,
}

impl MyFuckingChildren {
    fn new(read_dir: fs::ReadDir) -> Self {
        MyFuckingChildren { read_dir }
    }
}

impl Iterator for MyFuckingChildren {
    type Item = io::Result<MyFuckingPath>;
    fn next(&mut self) -> Option<io::Result<MyFuckingPath>> {
        self.read_dir.next().map(|result_dir_entry| {
            let dir_entry = result_dir_entry?;
            MyFuckingPath::new(dir_entry.path())
        })
    }
}
