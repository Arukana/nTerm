use ::pty;

#[derive(Default, Debug, Clone)]
pub struct Display {
    screen: Vec<(pty::Character, pty::Character)>,
    width: usize,
}

impl Display {
    pub fn get_width(&self) -> usize {
        self.width
    }
}

impl From<Vec<(pty::Character, pty::Character)>> for Display {
    fn from(screen: Vec<(pty::Character, pty::Character)>) -> Display {
        Display {
            screen: screen,
            width: 0,
        }
    }
}

impl From<(usize, Vec<(pty::Character, pty::Character)>)> for Display {
    fn from((width, screen): (usize, Vec<(pty::Character, pty::Character)>)) -> Display {
        Display {
            screen: screen,
            width: width,
        }
    }
}


impl<'a> IntoIterator for &'a Display {
    type Item = &'a [(pty::Character, pty::Character)];
    type IntoIter = ::std::slice::Chunks<'a, (pty::Character, pty::Character)>;

    fn into_iter(self) -> Self::IntoIter {
        if self.width.ne(&0) {
            self.screen.as_slice().chunks(self.width)
        } else {
            self.screen.as_slice().chunks(80)
        }
    }
}
