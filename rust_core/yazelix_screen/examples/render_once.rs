use std::io::{self, Write};

use yazelix_screen::{MandelbrotAnimation, ScreenAnimationContext, ScreenFrameProducer};

fn main() -> io::Result<()> {
    let context = ScreenAnimationContext {
        resolved_width: 80,
        resolved_height: 24,
        inner_width: 80,
        size_class: "wide",
    };
    let animation = MandelbrotAnimation::new(context);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in animation.render_frame() {
        if let Err(error) = writeln!(stdout, "{line}") {
            if error.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(error);
        }
    }

    Ok(())
}
