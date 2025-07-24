use schemars::JsonSchema;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BaseXMap {
    /// # Base2
    Base2([char; 2]),
    /// # Base4
    Base4([char; 4]),
    /// # Base8
    Base8([char; 8]),
    /// # Base16
    Base16([char; 16]),
    /// # Base32
    Base32([char; 32]),
    /// # Base64
    Base64(Vec<char>),
}

impl BaseXMap {
    pub fn encode(&self, input: &[u8]) -> String {
        if input.is_empty() {
            return String::new();
        }

        let (base, chars) = match self {
            Self::Base2(chars) => (2u8, chars.as_slice()),
            Self::Base4(chars) => (4u8, chars.as_slice()),
            Self::Base8(chars) => (8u8, chars.as_slice()),
            Self::Base16(chars) => (16u8, chars.as_slice()),
            Self::Base32(chars) => (32u8, chars.as_slice()),
            Self::Base64(chars) => (64u8, chars.as_slice()),
        };

        // Convert input bytes to big integer representation using Vec<u8>
        let mut digits = input.to_vec();

        // Handle zero case
        if digits.iter().all(|&x| x == 0) {
            return chars[0].to_string();
        }

        let mut result = Vec::new();

        // Repeatedly divide by base to get digits
        while !digits.iter().all(|&x| x == 0) {
            let mut carry = 0u16;

            // Divide the big integer by base
            #[allow(clippy::cast_possible_truncation)]
            for digit in &mut digits {
                let temp = carry * 256 + u16::from(*digit);
                // SAFETY: temp / base is always <= 255 since temp = carry * 256 + digit
                // where carry < base, so temp / base < 256, fits in u8
                *digit = (temp / u16::from(base)) as u8;
                carry = temp % u16::from(base);
            }

            result.push(chars[carry as usize]);

            // Remove leading zeros
            while digits.len() > 1 && digits[0] == 0 {
                digits.remove(0);
            }
        }

        result.reverse();
        result.into_iter().collect()
    }

    pub fn decode(&self, input: &str) -> Result<Vec<u8>, Error> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let (base, chars) = match self {
            Self::Base2(chars) => (2u8, chars.as_slice()),
            Self::Base4(chars) => (4u8, chars.as_slice()),
            Self::Base8(chars) => (8u8, chars.as_slice()),
            Self::Base16(chars) => (16u8, chars.as_slice()),
            Self::Base32(chars) => (32u8, chars.as_slice()),
            Self::Base64(chars) => (64u8, chars.as_slice()),
        };

        // Convert characters to indices
        let mut indices = Vec::new();
        for ch in input.chars() {
            match chars.iter().position(|&c| c == ch) {
                // SAFETY: idx is from chars array position, max 63 for Base64, fits in u8
                #[allow(clippy::cast_possible_truncation)]
                Some(idx) => indices.push(idx as u8),
                None => return Err(Error::InvalidCharacter(ch)),
            }
        }

        // Convert from base-X to bytes using big integer arithmetic
        let mut result = vec![0u8];

        for &digit in &indices {
            // Multiply result by base
            let mut carry = 0u16;
            for byte in result.iter_mut().rev() {
                let temp = u16::from(*byte) * u16::from(base) + carry;
                *byte = u8::try_from(temp & 0xFF).unwrap_or(0);
                carry = temp >> 8;
            }
            while carry > 0 {
                result.insert(0, u8::try_from(carry & 0xFF).unwrap_or(0));
                carry >>= 8;
            }

            // Add digit
            let mut carry = u16::from(digit);
            for byte in result.iter_mut().rev() {
                let temp = u16::from(*byte) + carry;
                *byte = u8::try_from(temp & 0xFF).unwrap_or(0);
                carry = temp >> 8;
            }
            while carry > 0 {
                result.insert(0, u8::try_from(carry & 0xFF).unwrap_or(0));
                carry >>= 8;
            }
        }

        // Remove leading zeros, but keep at least one byte
        while result.len() > 1 && result[0] == 0 {
            result.remove(0);
        }

        Ok(result)
    }

    pub fn decode_string(&self, input: &str) -> Result<String, Error> {
        let bytes = self.decode(input)?;
        Ok(String::from_utf8(bytes)?)
    }
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("字符 '{0}' 无效喵")]
    InvalidCharacter(char),
    #[error("长度无效喵")]
    InvalidLength(usize),
    #[error("UTF-8 编码错误喵")]
    Utf8Decode(#[from] std::string::FromUtf8Error),
}
