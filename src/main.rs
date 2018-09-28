use std::env;
use std::fs::{self, Metadata};
use std::io::{self, Error, ErrorKind};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in &args[1..] {
        tree(PathBuf::from(arg));
    }
}

fn tree(root: PathBuf) {
    let pathy = MyFuckingPath::new(root).unwrap();
    pathy.rprint().unwrap();
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum FileType {
    File,
    Folder,
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
    fn new(path: PathBuf) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;
        let err_body = Error::new(ErrorKind::Other, "WTF is this?");
        let file_type = FileType::new(metadata).ok_or(err_body)?;

        Ok(MyFuckingPath {
            path: path,
            file_type: file_type,
        })
    }

    fn print(&self) -> io::Result<()> {
        let printable_path = self.path.to_string_lossy();
        match self.file_type {
            File => {
                print!("{}", printable_path);
            }
            Folder => {
                print!("{}", printable_path);
                if !printable_path.ends_with("/") {
                    print!("/");
                }
            }
            SymLink => {
                let target = self.path.read_link()?;
                print!("{} -> {}", printable_path, target.to_string_lossy());
            }
        }
        println!("");
        Ok(())
    }

    fn rprint(&self) -> io::Result<()> {
        self.print()?;

        if let Folder = self.file_type {
            let mut children: Vec<MyFuckingPath> = self.children()?.collect::<io::Result<_>>()?;
            children.sort();
            for child in children.iter() {
                child.rprint()?;
            }
        }

        Ok(())
    }

    fn children(&self) -> io::Result<MyFuckingChildren> {
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
