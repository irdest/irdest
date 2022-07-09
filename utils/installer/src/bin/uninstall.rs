use colored::Colorize;
use installer::{Directories, File, Status};
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
    println!("{}", welcome.yellow());

    println!("This programm will uninstalsrl Ratman from your system!");

    let dirs = Directories::new();

    //////////

    let ratmand = File::Ratmand;
    let ratcat = File::Ratcat;
    let ratctl = File::Ratctl;
    let ratmand_man = File::RatmandMan;
    let systemd_unit = File::SystemdUnit;

    /////////

    ratmand.uninstall_state(&dirs);

    let ratmand_state = ratmand.uninstall_state(&dirs);
    let ratcat_state = ratcat.uninstall_state(&dirs);
    let ratctl_state = ratctl.uninstall_state(&dirs);
    let ratmand_man_state = ratmand_man.uninstall_state(&dirs);
    let systemd_unit_state = systemd_unit.uninstall_state(&dirs);

    let answer = Question::new("Do you want to proceed?")
        .default(Answer::YES)
        .show_defaults()
        .confirm();

    if answer == Answer::YES {
        skip_missing(ratmand_state, ratmand, &dirs);
        skip_missing(ratcat_state, ratcat, &dirs);
        skip_missing(ratctl_state, ratctl, &dirs);
        skip_missing(ratmand_man_state, ratmand_man, &dirs);
        skip_missing(systemd_unit_state, systemd_unit, &dirs);

        println!("{}", "Operation complete!".bright_green());
    } else {
        println!("{}", "Cancelled operation".bright_yellow())
    }
}

fn skip_missing(status: Status, file: File, dirs: &Directories) {
    if status == Status::Exists {
        file.uninstall(dirs)
    } else {
        println!(
            "Uninstall {}: {}",
            installer::print_path(&file.get_target(dirs)),
            "SKIP".yellow()
        )
    }
}
