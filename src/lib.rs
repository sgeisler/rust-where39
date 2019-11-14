#![no_std]

extern crate core;
extern crate libm;

mod wordlist;

use core::convert::TryInto;
use libm::F64Ext;
use wordlist::WORDLIST;

const TILES_SIZES: &[f64] = &[5.625, 0.125, 0.00277777777, 0.00006172839, 0.00000308642];
const RASTER_SIZE: &[(usize, usize)] = &[(32, 64), (45, 45), (45, 45), (45, 45), (20, 20)];

fn get_word_idx<'a>(word: &'a str) -> Result<usize, Error<'a>> {
    WORDLIST.iter()
        .enumerate()
        .find(|(_, w)| **w == word)
        .map(|(idx, _)| idx)
        .ok_or(Error::InvalidWord(word))
}

pub fn from_words<'a, I, S>(words: I) -> Result<(f64, f64), Error<'a>>
where I: Iterator<Item=&'a S> + ExactSizeIterator,
      S: AsRef<str> + 'a + ?Sized
{
    if words.len() > 5 {
        return Err(Error::TooManyWords(words.len()))
    }

    let (lat, lng) = words.enumerate()
        .map(|(idx, word)| {
            let word_idx_limit = RASTER_SIZE[idx].0 * RASTER_SIZE[idx].1;
            let word_idx = get_word_idx(word.as_ref())?;

            if word_idx >= word_idx_limit {
                return Err(Error::InvalidWord(word.as_ref()));
            }

            let lat_idx = word_idx / RASTER_SIZE[idx].1;
            let long_idx = word_idx % RASTER_SIZE[idx].1;

            Ok((
                (lat_idx as f64) * TILES_SIZES[idx],
                (long_idx as f64) * TILES_SIZES[idx],
            ))
        })
        .try_fold((-90f64, -180f64), |acc, coords| {
            match coords {
                Ok((lat, long)) => Ok((acc.0 + lat, acc.1 + long)),
                Err(e) => Err(e)
            }
        })?;

    Ok((lat, lng))
}

fn to_words(lat: f64, lng: f64) -> Result<[&'static str; 5], Error<'static>> {
    if (-90f64..=90f64).contains(&lat) == false || (-180f64..=180f64).contains(&lng) == false {
        return Err(Error::CoordinatesOutOfRange)
    }

    let mut lat = lat + 90f64;
    let mut lng = ((lng - 180f64) % 360f64) + 360f64;

    let mut code = [""; 5];

    for (idx, code_word) in code.iter_mut().enumerate() {
        let lat_idx_f64 = (lat / TILES_SIZES[idx]).floor();
        let lng_idx_f64 = (lng / TILES_SIZES[idx]).floor();

        lat -= lat_idx_f64 * TILES_SIZES[idx];
        lng -= lng_idx_f64 * TILES_SIZES[idx];

        let word_idx = (lat_idx_f64 as usize) * RASTER_SIZE[idx].1 + (lng_idx_f64 as usize);
        *code_word = WORDLIST[word_idx];
    }

    Ok(code)
}

#[derive(Debug)]
pub enum Error<'a> {
    InvalidWord(&'a str),
    TooManyWords(usize),
    CoordinatesOutOfRange
}

#[cfg(test)]
mod tests {
    use crate::to_words;

    #[test]
    fn test_words_to_coords() {
        let words = &["slush", "battle", "damage", "dentist"][..];

        let (lat, lng) = super::from_words(words.iter()).unwrap();
        assert_eq!(lat, 51.02561728383f64);
        assert_eq!(lng, 13.72333333297f64);
    }

    #[test]
    fn test_coords_to_words() {
        let code = to_words(51.02561728383f64, 13.72333333297f64).unwrap();
        let expected = &["slush", "battle", "damage", "dentist"][..];

        assert_eq!(expected, &code[..4]);
    }
}