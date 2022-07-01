use installer::{Directories, File};

fn main() {
    let welcome = "
  ██████╗  █████╗ ████████╗███╗   ███╗ █████╗ ███╗   ██╗
  ██╔══██╗██╔══██╗╚══██╔══╝████╗ ████║██╔══██╗████╗  ██║
  ██████╔╝███████║   ██║   ██╔████╔██║███████║██╔██╗ ██║
  ██╔══██╗██╔══██║   ██║   ██║╚██╔╝██║██╔══██║██║╚██╗██║
  ██║  ██║██║  ██║   ██║   ██║ ╚═╝ ██║██║  ██║██║ ╚████║
  ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝

";
    println!("{}", welcome);

    println!("This programm will uninstall Ratman from your system!");

    let dirs = Directories::new();

    //////////

    // let ratmand = File::Ratmand.uninstall(&dirs);
    // let ratcat = File::Ratcat.uninstall(&dirs);
    // let ratctl = File::Ratctl.uninstall(&dirs);
    // let ratmand_man = File::RatmandMan.uninstall(&dirs);
    // let systemd_unit = File::SystemdUnit.uninstall(&dirs);

    /////////
}
