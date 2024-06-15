use argh::FromArgs;
use linicon::{IconPath, IconType, LiniconError, Theme};
use prettytable::{format::consts::FORMAT_CLEAN, row, Table};
use std::env;
use std::path::PathBuf;

#[derive(Debug, FromArgs)]
/// Get icons
struct Args {
    /// names of the icons to get
    #[argh(positional)]
    names: Vec<String>,

    /// icon size
    #[argh(option, short = 's')]
    size: Option<u16>,

    /// icon scale
    #[argh(option, short = 'c')]
    scale: Option<u16>,

    /// desired file formats (allowed: png, svg, xmp)
    #[argh(option, short = 'x')]
    formats: Option<String>,

    /// theme to get icons from
    #[argh(option, short = 't')]
    theme: Option<String>,

    /// list installed themes
    #[argh(switch, short = 'L')]
    list_themes: bool,

    /// show more information
    #[argh(switch, short = 'l')]
    long: bool,

    /// print the user's current theme
    #[argh(switch, short = 'U')]
    print_user_theme: bool,

    /// print the program version
    #[argh(switch)]
    version: bool,
}

fn main() {
    let args: Args = argh::from_env();
    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
    } else if args.list_themes {
        list_themes(args.long)
    } else if args.print_user_theme {
        print_user_theme()
    } else {
        let formats = get_formats(args.formats.as_ref());
        // get the icons for each theme
        let res: Vec<_> = args
            .names
            .iter()
            .map(|name| get_icons_with_fallback(name, &args, &formats))
            .collect();
        if args.long {
            print_icons_long(&res);
        } else {
            print_icons(&res);
        }
        print_errors(&res);
    }
}

fn print_errors(res: &[(Vec<IconPath>, Option<LiniconError>)]) {
    let mut has_errors = false;
    for (_, errors) in res {
        if let Some(error) = errors {
            has_errors = true;
            eprintln!("Error: {}", error);
        }
    }
    if has_errors {
        std::process::exit(1);
    }
}

fn print_icons(res: &[(Vec<IconPath>, Option<LiniconError>)]) {
    for (icons, _) in res {
        for icon in icons {
            println!("{}", icon.path.display());
        }
    }
}

fn print_icons_long(res: &[(Vec<IconPath>, Option<LiniconError>)]) {
    let mut table = Table::new();
    table.set_format(*FORMAT_CLEAN);
    table.set_titles(row![
        "Path", "Theme", "Type", "Min size", "Max size", "Scale"
    ]);
    for (icons, _) in res {
        for icon in icons {
            let format = match icon.icon_type {
                IconType::PNG => "png",
                IconType::SVG => "svg",
                IconType::XMP => "xmp",
            };
            table.add_row(row![
                icon.path.display(),
                icon.theme,
                format,
                icon.min_size,
                icon.max_size,
                icon.scale
            ]);
        }
    }
    table.printstd();
}

fn get_formats(formats_str: Option<&String>) -> Option<Vec<IconType>> {
    formats_str
        .map(|s| s.split(',')
        .map(|s| match s.to_lowercase().as_str() {
            "png" => IconType::PNG,
            "svg" => IconType::SVG,
            "xmp" => IconType::XMP,
            unsupported => {
                eprintln!("Invalid icon type {}. Supported formats are png, svg, and xmp.", unsupported);
                std::process::exit(1);
            }
        }).collect())
}

fn print_user_theme() {
    if let Some(name) = linicon::get_system_theme() {
        println!("{}", name);
    } else {
        eprintln!("Error: Couldn't get user's icon theme");
        std::process::exit(1);
    }
}

fn list_themes(long: bool) {
    let mut themes = linicon::themes();
    themes.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    if long {
        print_themes_long(themes)
    } else {
        print_themes(&themes)
    }
}

fn print_themes(themes: &[Theme]) {
    for theme in themes {
        println!("{}", theme.name);
    }
}

fn print_themes_long(themes: Vec<Theme>) {
    let mut table = Table::new();
    table.set_format(*FORMAT_CLEAN);
    table.set_titles(row![
        "Name",
        "Inherits",
        "Display name",
        "Comment",
        "Paths",
    ]);
    for theme in themes {
        let inherits = fmt_list(&theme.inherits.unwrap_or_default());
        let paths: Vec<_> = theme
            .paths
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        let paths = fmt_list(&paths);
        table.add_row(row![
            theme.name,
            inherits,
            theme.display_name,
            theme.comment.unwrap_or_default(),
            paths,
        ]);
    }
    table.printstd();
}

fn get_icons_with_fallback(
    icon_name: &str,
    args: &Args,
    formats: &Option<Vec<IconType>>,
) -> (Vec<IconPath>, Option<LiniconError>) {
    // Try with no-fallbacks first
    let mut result = get_icons(icon_name, args, formats, true);
    // If no icons found, try again with fallbacks
    if result.0.is_empty() {
        result = get_icons(icon_name, args, formats, false);
    }
    result
}

fn get_icons(
    icon_name: &str,
    args: &Args,
    formats: &Option<Vec<IconType>>,
    no_fallbacks: bool,
) -> (Vec<IconPath>, Option<LiniconError>) {
    let mut iter = linicon::lookup_icon(icon_name);
    // Set lookup params
    if let Some(size) = args.size {
        iter = iter.with_size(size);
    }
    if let Some(scale) = args.scale {
        iter = iter.with_scale(scale);
    }
    if let Some(theme) = &args.theme {
        iter = iter.from_theme(theme);
    }
    iter = iter.use_fallback_themes(!no_fallbacks);

    // Add user-specific icon directories to the search path
    let home_dir = env::var("HOME").expect("Could not get home directory");
    let user_icon_paths: Vec<String> = vec![
        format!("{}/.icons", home_dir),
        format!("{}/.local/share/icons", home_dir),
    ];

    iter = iter
        .with_search_paths(&user_icon_paths)
        .expect("Failed to add search paths");

    let mut error: Option<LiniconError> = None;
    let mut icons: Vec<IconPath> = vec![];

    for icon_result in iter {
        match icon_result {
            Ok(icon) => {
                if let Some(formats) = &formats {
                    if formats.contains(&icon.icon_type) {
                        icons.push(icon);
                        break; // Found a valid icon, exit the loop
                    }
                } else {
                    icons.push(icon);
                    break; // Found a valid icon, exit the loop
                }
            }
            Err(e) => {
                error = Some(e);
                break; // Exit on the first error
            }
        }
    }

    (icons, error)
}

fn fmt_list(list: &[String]) -> String {
    let mut out = String::new();
    let mut first = true;
    for item in list {
        if !first {
            out.push(',');
        } else {
            first = false;
        }
        out.push_str(item);
    }
    out
}
