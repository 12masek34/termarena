use crate::map::Map;


pub fn render_map(map: &Map) {
    for row in &map.tiles {
        let line: String = row.iter().collect();
        println!("{}", line);
    }
}
