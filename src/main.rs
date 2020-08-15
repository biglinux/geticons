use argh::FromArgs;
use linicon::{IconPath, IconType};
use std::cmp::Ordering;

#[derive(Debug, FromArgs)]
/// get icons
struct Args {
    #[argh(positional)]
    names: Vec<String>,

    /// icon size
    #[argh(option, short = 's')]
    size: Option<u16>,

    /// icon scale
    #[argh(option, short = 'c')]
    scale: Option<u16>,

    /// format
    #[argh(option, short = 'x')]
    formats: Option<String>,

    /// theme name
    #[argh(option, short = 't')]
    theme: Option<String>,

    /// no fallback
    #[argh(switch)]
    no_fallbacks: bool,

    /// list installed themes
    #[argh(switch, short = 'L')]
    list_themes: bool,

    /// show more information
    #[argh(switch, short = 'l')]
    long: bool,
}

fn main() {
    let args: Args = argh::from_env();
    if args.list_themes {
        for theme in linicon::themes() {
            println!("{}", theme.name);
        }
        return;
    }
    let formats: Option<Vec<_>> = args.formats.as_ref().map(|s| s.split(',').map(|s| match s.to_lowercase().as_str() {
                "png" => IconType::PNG,
                "svg" => IconType::SVG,
                "xmp" => IconType::XMP,
                unsupported => {
                    eprintln!("Invalid icon type {}.  Supported formats are png, svg, and xmp.", unsupported);
                    std::process::exit(1);
                }
        }).collect());
    for icon_name in &args.names {
        get_icons(icon_name, &args, &formats);
    }
}

fn get_icons(icon_name: &str, args: &Args, formats: &Option<Vec<IconType>>) {
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
        Some(formats) => partition_by_theme(iter.filter(|icon| formats.contains(&icon.icon_type))),
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
    for icons in themes {
        for icon in icons {
            println!("{}", icon.path.display());
        }
    }
}

/// Splits icon list into one list per theme
fn partition_by_theme(iter: impl Iterator<Item = IconPath>) -> Vec<Vec<IconPath>> {
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

