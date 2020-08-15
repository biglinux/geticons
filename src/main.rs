use argh::FromArgs;
use linicon::{IconPath, IconType};
use prettytable::{cell, format::consts::FORMAT_CLEAN, row, Table};
use std::cmp::Ordering;

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

    /// don't go to fallback themes
    #[argh(switch)]
    no_fallbacks: bool,

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
        let mut themes = linicon::themes();
        themes.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        if args.long {
            for theme in themes {
                println!("{}", theme.name);
            }
        } else {
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
                let inherits = fmt_list(&theme.inherits.unwrap_or(Vec::new()));
                let paths = theme
                    .paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect();
                let paths = fmt_list(&paths);
                table.add_row(row![
                    theme.name,
                    inherits,
                    theme.display_name,
                    theme.comment.unwrap_or(String::new()),
                    paths,
                ]);
            }
            table.printstd();
        }
    } else if args.print_user_theme {
        if let Some(name) = linicon::get_system_theme() {
            println!("{}", name);
            return;
        } else {
            eprintln!("Error: Couldn't get user's icon theme");
            std::process::exit(1);
        }
    } else {
        let formats: Option<Vec<_>> = args.formats.as_ref().map(|s| s.split(',').map(|s| match s.to_lowercase().as_str() {
                "png" => IconType::PNG,
                "svg" => IconType::SVG,
                "xmp" => IconType::XMP,
                unsupported => {
                    eprintln!("Invalid icon type {}.  Supported formats are png, svg, and xmp.", unsupported);
                    std::process::exit(1);
                }
        }).collect());
        // get the icons for each theme
        let icons: Vec<IconPath> = args
            .names
            .iter()
            .flat_map(|name| get_icons(name, &args, &formats))
            .collect();
        if args.long {
            let mut table = Table::new();
            table.set_format(*FORMAT_CLEAN);
            table.set_titles(row![
                "Path", "Theme", "Type", "Min size", "Max size", "Scale"
            ]);
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
            table.printstd();
        } else {
            for icon in icons {
                println!("{}", icon.path.display());
            }
        }
    }
}

fn get_icons(
    icon_name: &str,
    args: &Args,
    formats: &Option<Vec<IconType>>,
) -> Vec<IconPath> {
    let mut iter = linicon::lookup_icon(icon_name);
    if let Some(size) = args.size {
        iter = iter.with_size(size);
    }
    if let Some(scale) = args.scale {
        iter = iter.with_scale(scale);
    }
    if let Some(theme) = &args.theme {
        iter = iter.from_theme(theme);
    }
    iter = iter.use_fallback_themes(!args.no_fallbacks);
    let iter = iter.filter(Result::is_ok).map(Result::unwrap);
    let mut themes = match &formats {
        Some(formats) => partition_by_theme(
            iter.filter(|icon| formats.contains(&icon.icon_type)),
        ),
        None => partition_by_theme(iter),
    };
    for icons in &mut themes {
        // SVGs first, them PNGs and XMPs by max size
        icons.sort_unstable_by(|a, b| {
            if a.icon_type == IconType::SVG {
                Ordering::Greater
            } else if b.icon_type == IconType::SVG {
                Ordering::Less
            } else {
                b.max_size.cmp(&a.max_size)
            }
        });
    }
    themes.into_iter().flatten().collect()
}

/// Splits icon list into one list per theme
fn partition_by_theme(
    iter: impl Iterator<Item = IconPath>,
) -> Vec<Vec<IconPath>> {
    let mut themes = Vec::new();
    let mut curr_theme = None;
    let mut curr_list = Vec::new();
    for icon in iter {
        if curr_theme.as_ref() != Some(&icon.theme) {
            curr_theme = Some(icon.theme.clone());
            themes.push(curr_list);
            curr_list = Vec::new();
        }
        curr_list.push(icon);
    }
    themes.push(curr_list);
    themes
}

fn fmt_list(list: &Vec<String>) -> String {
    let mut out = String::new();
    let mut first = true;
    for item in list {
        if !first {
            out.push(',');
        } else {
            first = false;
        }
        out.push_str(&item);
    }
    out
}
