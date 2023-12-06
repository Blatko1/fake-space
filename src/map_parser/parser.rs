use super::error::{DimensionsError, ParseError};

struct MapParser<'a> {
data: &'a str,

//
dimensions: (u32, u32),
lvl1: Option<u32>,
lvl2: Option<u32>,
lvl3: Option<u32>,
lvl4: Option<u32>,
}

impl<'a> MapParser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            data,
            dimensions: (0, 0),
            lvl1: None,
            lvl2: None,
            lvl3: None,
            lvl4: None,
            
        }
    }
    fn parse(mut self) -> Result<(), ParseError> {
        let mut lines = self.data
            .lines()
            .enumerate()
            .map(|(i, line)| (i as u32, line.split("//").next().unwrap().trim()))
            .filter(|(_, line)| !line.is_empty());

        if let Some((i, dimensions_str)) = lines.next() {
            match self.parse_dimensions(dimensions_str) {
                Ok(_) => (),
                Err(e) => return Err(ParseError::Dimensions(e, i)),
            }
        } else {
            return Err(ParseError::Invalid)
        }
    
        for (i, line) in lines {
            let key = line.chars().next().unwrap();
            match key {
                '*' => (),
                '#' => (),
                '$' => (),
                '_' => (),
                k if k.is_ascii_digit() => (),
                _ => panic!("Unknown line key {}", key)
            }
        }

        Ok(())
    }

    fn parse_dimensions(&mut self, line: &str) -> Result<(), DimensionsError> {
        let mut split: Vec<_> = line.split('x').collect();
        if split.len() != 2 {
            return Err(DimensionsError::InvalidFormat(line.to_owned()))
        }
        let Ok(d1) = split[0].trim().parse::<u32>() else {
            return Err(DimensionsError::ParseError(split[0].to_owned()))
        };
        let Ok(d2) = split[1].trim().parse::<u32>() else {
            return Err(DimensionsError::ParseError(split[1].to_owned()))
        };

        if d1 == 0 || d2 == 0 {
            return Err(DimensionsError::IllegalDimensions(d1, d2))
        }

        self.dimensions = (d1, d2);
        Ok(())
    }
}

#[test]
fn parsing() {
    let input = include_str!("../../maps/new_syntax.txt");
    MapParser::new(input).parse();
}