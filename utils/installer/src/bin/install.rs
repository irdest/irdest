use colored::Colorize;
use installer::{Directories, File};
use question::{Answer, Question};

fn main() {
    let welcome = "
  ██████╗  █████╗ ████████╗███╗   ███╗ █████╗ ███╗   ██╗
  ██╔══██╗██╔══██╗╚══██╔══╝████╗ ████║██╔══██╗████╗  ██║
  ██████╔╝███████║   ██║   ██╔████╔██║███████║██╔██╗ ██║
  ██╔══██╗██╔══██║   ██║   ██║╚██╔╝██║██╔══██║██║╚██╗██║
  ██║  ██║██║  ██║   ██║   ██║ ╚═╝ ██║██║  ██║██║ ╚████║
  ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝

";
    println!("{}", welcome.bright_purple());

    println!("Hello!\nThis program will determine how to install ratman on your system...\n");

    let dirs = Directories::new();

    //////////

    let ratmand = File::Ratmand;
    let ratcat = File::Ratcat;
    let ratctl = File::Ratctl;
    let ratmand_man = File::RatmandMan;
    let systemd_unit = File::SystemdUnit;

    /////////

    ratmand.install_state(&dirs);
    ratcat.install_state(&dirs);
    ratctl.install_state(&dirs);
    ratmand_man.install_state(&dirs);
    systemd_unit.install_state(&dirs);

    let answer = Question::new("Do you want to proceed?")
        .default(Answer::YES)
        .show_defaults()
        .confirm();

    if answer == Answer::YES {
        let bundle_dir = installer::bundle_dir();

        ratmand.install(&dirs, &bundle_dir);
        ratcat.install(&dirs, &bundle_dir);
        ratctl.install(&dirs, &bundle_dir);
        ratmand_man.install(&dirs, &bundle_dir);
        systemd_unit.install_unitfile(&dirs, &bundle_dir);

        println!("{}", "Operation complete!".bright_green());
    } else {
        println!("{}", "Cancelled operation".bright_yellow())
    }
}
