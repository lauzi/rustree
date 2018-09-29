use std::borrow::Cow;
use std::fs::{self, Metadata};
use std::io::{self, Error, ErrorKind};
use std::iter::Peekable;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

extern crate clap;
use clap::{App, Arg};

extern crate colored;
use colored::*;

fn main() {
    let matches = App::new("rustree")
        .about("Shity tree")
        .author("Nozh")
        .version("0.1")
        .arg(Arg::with_name("show-dot-files").short("a").long("all"))
        .arg(Arg::with_name("dirs-only").short("d").long("dir"))
        .arg(Arg::with_name("no-color").long("no-color"))
        .arg(Arg::with_name("paths").multiple(true).default_value("."))
        .get_matches();

    let mut printer = MyFuckingPrinter::new();
    printer.show_dot_files = matches.is_present("show-dot-files");
    printer.dirs_only = matches.is_present("dirs-only");
    printer.colored = !matches.is_present("no-color");

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
    dirs_only: bool,
    colored: bool,
}

impl MyFuckingPrinter {
    fn new() -> Self {
        MyFuckingPrinter {
            bar: vec![],
            is_last: false,
            show_dot_files: false,
            dirs_only: false,
            colored: true,
        }
    }

    fn p(&self, path: &MyFuckingPath) -> bool {
        if !self.show_dot_files && path.is_dot_file() {
            return false;
        }

        if self.dirs_only && path.file_type != Folder {
            return false;
        }

        true
    }

    fn get_children(&self, path: MyFuckingPath) -> io::Result<Vec<MyFuckingPath>> {
        let p = |path: &MyFuckingPath| self.p(path);
        let mut children: Vec<MyFuckingPath> = path
            .children()?
            .filter(|r_path| r_path.as_ref().map(p).unwrap_or(true))
            .collect::<io::Result<_>>()?;
        children.sort_by(|ref a, ref b| a.path.cmp(&b.path));
        Ok(children)
    }

    fn path_color(&self, path: &MyFuckingPath) -> &str {
        if !self.colored {
            "white"
        } else if let Folder = path.file_type {
            "blue"
        } else if let SymLink = path.file_type {
            "magenta" // purple
        } else if path.is_exec() {
            "red"
        } else {
            "white"
        }
    }

    fn print_path(&mut self, path: &MyFuckingPath) -> io::Result<()> {
        self.print_tree_bars();

        fn end_with_slash(s: &str) -> String {
            if s.ends_with("/") {
                String::from(s)
            } else {
                format!("{}/", s)
            }
        }

        let mut cur_path = Cow::Borrowed(path);
        // TODO: refactor this ball of shit
        // TODO: add switch to turn off symlink following
        loop {
            {
                // Block to end name's life
                let name = cur_path.printable_name();
                let color = self.path_color(&cur_path);
                match cur_path.file_type {
                    File => {
                        println!("{}", name.color(color));
                        return Ok(());
                    }
                    Folder => {
                        println!("{}", end_with_slash(&name).color(color));
                        return Ok(());
                    }
                    SymLink => {
                        print!("{} -> ", name.color(color));
                    }
                }
            }

            // Fallthrough: SymLink
            let link_content = cur_path.path.read_link()?;
            let mut target = PathBuf::from(&cur_path.path);
            target.pop(); // pop the SymLink name
            target.push(&link_content);
            match MyFuckingPath::new(target) {
                Ok(new_path) => {
                    // TODO: cycle detection
                    cur_path = Cow::Owned(new_path);
                }
                Err(ref err) if err.kind() == ErrorKind::NotFound => {
                    println!("{}", link_content.to_string_lossy());
                    return Ok(());
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    fn rustree(&mut self, path: MyFuckingPath) -> io::Result<()> {
        self.print_path(&path)?;

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum FileType {
    Folder,
    File,
    SymLink,
}
use FileType::*;

impl FileType {
    fn new(metadata: &Metadata) -> Option<Self> {
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

#[derive(Clone)]
struct MyFuckingPath {
    file_type: FileType,
    path: PathBuf,
    metadata: Metadata,
}

impl MyFuckingPath {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;
        let err_body = Error::new(ErrorKind::Other, "WTF is this?");
        let file_type = FileType::new(&metadata).ok_or(err_body)?;

        Ok(MyFuckingPath {
            path: path,
            file_type: file_type,
            metadata: metadata,
        })
    }

    pub fn is_dot_file(&self) -> bool {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().starts_with("."))
            .unwrap_or(false)
    }

    pub fn is_exec(&self) -> bool {
        const EXEC_BITS: u32 = 0o111;

        let mode = self.metadata.permissions().mode();
        mode & EXEC_BITS != 0
    }

    fn printable_name(&self) -> Cow<str> {
        self.path
            .file_name()
            .unwrap_or_else(|| self.path.as_os_str())
            .to_string_lossy()
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
