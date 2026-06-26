use pincers::pincers;

// Define dummy types used in the macro expansion (Button, Label) – they are provided by the macro itself.
// We'll just use the macro in a const.
const MAIN_WINDOW: pincers::PincersWindow = pincers! {
    Button.text("Zaloguj").padding(4).id(1);
    Label.text("Sukces!").visible(false).id(2);
};

fn main() {
    // Just ensure the type exists and has a method.
    let _ = MAIN_WINDOW;
}
