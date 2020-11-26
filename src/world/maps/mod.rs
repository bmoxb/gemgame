struct Map {}

trait Generator {
    fn generate(&self) -> Map { Map {} }
}

struct OverworldGenerator {}
impl Generator for OverworldGenerator {}