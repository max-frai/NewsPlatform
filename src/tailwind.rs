use duct::cmd;

pub async fn process_tailwind() -> std::io::Result<String> {
    let mut css_container = String::new();
    let modules_dir = "templates/modules/";

    for entry in std::fs::read_dir(modules_dir)? {
        let entry = entry?;
        let path = format!("{}/tpl.scss", entry.path().as_os_str().to_str().unwrap());
        let css = std::fs::read_to_string(path)?;

        css_container = format!("{}\n{}", css_container, css);
    }

    let main_css = std::fs::read_to_string("templates/css/main.scss")?;
    let all_css = format!("{}\n{}", main_css, css_container);

    std::fs::write("templates/css/main.css", all_css)?;

    cmd!("postcss", "templates/css/main.css", "--replace").read()
}
