use super::entities::Entity;

pub struct Map {
    /// Tiles that this map is comprised of.
    tiles: Vec<Tile>, // Use array & const generics when latter is stable!
    /// Width of this map in tiles.
    width: usize,
    /// Height of this map in tiles.
    height: usize,
    /// Entities currently on this map.
    entities: Vec<Entity>
}

impl Map {
    /// Get a reference to the tile at the given coordinates.
    pub fn at(&self, x: usize, y: usize) -> Option<&Tile> {
        let option_tile = self.tiles.get(y * self.width + x);

        if option_tile.is_none() {
            log::trace!("No tile found on map at coordinates: {}, {}", x, y);
        }

        option_tile
    }

    /// Check if a given set of coordinates are within bounds for this map.
    pub fn within_bounds(&self, x: usize, y: usize) -> bool {
        return x <= self.width && y <= self.height;
    }
}

pub struct Tile {
    tile_type: TileType,
    blocking: bool
}

enum TileType {}

trait Generator {
    fn generate(&self, seed: u32) -> Map;
}

struct OverworldGenerator {}
//impl Generator for OverworldGenerator {}

#[cfg(test)]
mod test {
    // TODO: ...
}