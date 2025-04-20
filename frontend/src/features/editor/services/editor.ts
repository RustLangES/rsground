import { dockview } from "@features/panels/stores";
import { setEditingFiles, setSyncFiles } from "../stores";

const CODE_EXAMPLE = `
pub struct Something {
  prop: String
}

#[tokio::main]
fn main<'a>() {
  let a = String::new();
  println!(r#"{a}"#);
}`;

export async function openFile(filepath: string) {
  const filename = filepath.split("/").pop();

  dockview().addPanel({ 
    id: `file:${filepath}`,
    component: "code",
    title: filename,
  })

  setEditingFiles(filepath, []);
  setSyncFiles(filepath, CODE_EXAMPLE);
}
