use std::borrow::Cow;
use std::fs::{self, Metadata};
use std::io::{self, Error, ErrorKind};
use std::iter::Peekable;
use std::path::PathBuf;

extern crate clap;
use clap::{App, Arg};

fn main() {
    let matches = App::new("rustree")
        .about("Shity tree")
        .author("Nozh")
        .version("0.1")
        .arg(Arg::with_name("show-dot-files").short("a").long("all"))
        .arg(Arg::with_name("paths").multiple(true).default_value("."))
        .get_matches();

    let mut printer = MyFuckingPrinter::new();
    printer.show_dot_files = matches.is_present("show-dot-files");

    let paths = matches.values_of("paths").unwrap();
    for path in paths {
        let pathy = MyFuckingPath::new(PathBuf::from(path)).unwrap();
        printer.rustree(pathy).unwrap();
    }
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
    show_dot_files: bool,
}

impl MyFuckingPrinter {
    fn new() -> Self {
        MyFuckingPrinter {
            bar: vec![],
            is_last: false,
            show_dot_files: false,
        }
    }

    fn get_children(&self, path: MyFuckingPath) -> io::Result<Vec<MyFuckingPath>> {
        let filter_children =
            |path: &MyFuckingPath| -> bool { self.show_dot_files || !path.is_dot_file() };
        let mut children: Vec<MyFuckingPath> = path
            .children()?
            .filter(|r_path| r_path.as_ref().map(filter_children).unwrap_or(true))
            .collect::<io::Result<_>>()?;
        children.sort();
        Ok(children)
    }

    fn rustree(&mut self, path: MyFuckingPath) -> io::Result<()> {
        self.print_tree_bars();
        println!("{}", path.summary()?);

        if let Folder = path.file_type {
            self.bar.push(true);
            let mut children = self.get_children(path)?;
            let mut is_last_iter: IsLastIterator<_> = children.drain(..).into();
            for (child, is_last) in is_last_iter {
                self.is_last = is_last;
                self.rustree(child)?;
            }
            self.bar.pop();
        }

        Ok(())
    }

    fn print_tree_bars(&mut self) {
        let mut s = String::from("");
        let is_last_iter: IsLastIterator<_> = self.bar.iter_mut().into();
        for (bar, is_last) in is_last_iter {
            s.push_str(
                match (*bar, is_last, self.is_last) {
                    (false, _, _) => Bar::X,
                    (true, false, _) => Bar::I,
                    (true, true, false) => Bar::T,
                    (true, true, true) => {
                        *bar = false;
                        Bar::L
                    }
                }.str(),
            )
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

    fn is_dot_file(&self) -> bool {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().starts_with("."))
            .unwrap_or(false)
    }

    fn printable_name(&self) -> Cow<str> {
        self.path
            .file_name()
            .unwrap_or_else(|| self.path.as_os_str())
            .to_string_lossy()
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

struct IsLastIterator<T: Iterator> {
    it: Peekable<T>,
}

impl<T: Iterator> IsLastIterator<T> {
    fn new(it: T) -> Self {
        IsLastIterator { it: it.peekable() }
    }
}

impl<T: Iterator> From<T> for IsLastIterator<T> {
    fn from(it: T) -> IsLastIterator<T> {
        IsLastIterator::new(it)
    }
}

impl<T: Iterator> Iterator for IsLastIterator<T> {
    type Item = (T::Item, bool);
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|item| {
            let is_last = self.it.peek().is_none();
            (item, is_last)
        })
    }
}
