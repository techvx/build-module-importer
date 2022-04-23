const TAB_SIZE: usize = 2;

fn main() {

  let tree = SourceCode::new();
  let src = tree.modularize();

  let root = std::fs::read_to_string("./src/main.rs")
    .expect("can't read the main.rs file");
  // avoid constant module insertions
  if root.starts_with(&src) { return };

  let moduled = format!("{}\r\n{}", src, root);
  std::fs::write("./src/main.rs", moduled)
    .expect("couldn't modify the main.rs");
}

type Str = &'static str;
type Boxed = Box<SourceCode>;

#[derive(Clone, Debug)]
struct SourceCode {
  directory: Str,
  modules: Vec<Str>,
  folders: Vec<Boxed>
}

impl SourceCode {
  fn new() -> Self {
    let empty = Self {
      directory: "./src",
      modules: Vec::new(),
      folders: Vec::new()
    }; empty.tree()
  }
  fn modularize(&self) -> String {
    let mut code = String::new();
    for m in self.modules.iter()
    .filter(|&&m| "main" != m) {
      let line = format!("pub mod {};\r\n", m);
      code.push_str(&line);
    }
    for f in &self.folders {
      let module_split = f.directory
        .split(|c| ['\\', '/'].contains(&c))
        .collect::<Vec<_>>();
      let space = " ".repeat(TAB_SIZE).repeat(module_split.len() - 2);
      let subcode = format!("pub mod {} {{ \r\n{}}}\r\n",
        module_split.last()
          .unwrap_or(&f.directory),
        f.modularize().lines()
          .map(|line| format!("{}{}", space, line))
          .fold(String::new(), |mut all_code, line| {
            let line_with_crlf = format!("{}\r\n", line);
            all_code.push_str(&line_with_crlf); 
            all_code
          })
      ); code.push_str(&subcode);
    }
    code
  }
  fn tree(mut self) -> Self {
  
    let mut dir = std::fs::read_dir(self.directory)
      .expect("directory issue");
      
    while let Some(Ok(e)) = dir.next() {

      let fname = e.file_name().into_string()
        .expect("file name");

      let ftype = e.file_type()
        .expect("file type");

      // another directory?
      if ftype.is_dir() {
        let new_dir = format!("{}/{}", self.directory, fname);
        let boxed_dir = Box::leak(Box::new(new_dir));
        let empty = Self {
          directory: boxed_dir,
          modules: Vec::new(), 
          folders: Vec::new()
        }; 
        let sub_tree = empty.tree();
        let boxed = Box::new(sub_tree);
        self.folders.push(boxed);
      }

      let is_rust_module = ftype.is_file()
        && fname.ends_with(".rs");

      if is_rust_module {
        let module_name = fname.trim_end_matches(".rs").to_string();
        let boxed_module = Box::leak(Box::new(module_name));
        self.modules.push(boxed_module);
      }
    }
    self
  }
}
