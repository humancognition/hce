use hce_core::{ChunkSpec, Hce, HceCase, HceMode, LanguageLevel, RecoveryResult, TsGranularity};
use hce_fpe::{build_cipher, CipherKind};
use pyo3::prelude::*;

#[pyclass(name = "Hce", from_py_object)]
#[derive(Clone)]
struct PyHce {
    inner: Hce,
}

#[pymethods]
impl PyHce {
    #[new]
    #[pyo3(signature = (key = None, level = "universal", mode = "sealed", bit_width = 128))]
    fn new(key: Option<&[u8]>, level: &str, mode: &str, bit_width: u32) -> PyResult<Self> {
        let hce_level = match level {
            "universal" => LanguageLevel::Universal,
            "eu" => LanguageLevel::Eu,
            "en" => LanguageLevel::En,
            "numeric" => LanguageLevel::Numeric,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "level must be one of: universal, eu, en, numeric",
                ))
            }
        };
        let hce_mode = match mode {
            "sealed" => HceMode::Sealed,
            "open" => HceMode::Open,
            "plain" => HceMode::Plain,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "mode must be one of: sealed, open, plain",
                ))
            }
        };
        if let Some(k) = &key {
            if k.is_empty() && hce_mode != HceMode::Plain {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "key must not be empty for sealed/open modes",
                ));
            }
        }
        Ok(PyHce {
            inner: Hce::new(key, hce_level, hce_mode).with_bit_width(bit_width),
        })
    }

    fn with_bit_width(&self, bits: u32) -> Self {
        PyHce {
            inner: self.inner.clone().with_bit_width(bits),
        }
    }

    fn with_modulus(&self, modulus: u128) -> PyResult<Self> {
        if modulus < 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "modulus must be at least 2",
            ));
        }
        Ok(PyHce {
            inner: self.inner.clone().with_modulus(modulus),
        })
    }

    #[pyo3(signature = (kind, key = None))]
    fn with_cipher(&self, kind: &str, key: Option<&[u8]>) -> PyResult<Self> {
        let ck = match kind {
            "feistel" | "feistel8" => CipherKind::Feistel8,
            "shuffle" | "shuffle4" => CipherKind::Shuffle4,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "unknown cipher: use 'feistel' or 'shuffle'",
                ))
            }
        };
        let fpe = build_cipher(ck, key, self.inner.domain());
        Ok(PyHce {
            inner: self.inner.clone().with_cipher(fpe),
        })
    }

    fn with_case(&self, case: &str) -> PyResult<Self> {
        let c = match case {
            "upper" => HceCase::Upper,
            "lower" => HceCase::Lower,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "case must be 'upper' or 'lower'",
                ))
            }
        };
        Ok(PyHce {
            inner: self.inner.clone().with_case(c),
        })
    }

    fn with_check_syllables(&self, n: usize) -> Self {
        PyHce {
            inner: self.inner.clone().with_check_syllables(n),
        }
    }

    fn with_separator(&self, sep: &str) -> PyResult<Self> {
        let ch = sep.chars().next().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("separator must not be empty")
        })?;
        if !hce_core::validate_separator(ch) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "invalid separator: must be non-alphabetic",
            ));
        }
        Ok(PyHce {
            inner: self
                .inner
                .clone()
                .with_chunk_spec(ChunkSpec::natural_with_separator(ch)),
        })
    }

    fn with_chunk_none(&self) -> Self {
        PyHce {
            inner: self.inner.clone().with_chunk_spec(ChunkSpec::none()),
        }
    }

    fn with_chunk_fixed(&self, char_size: usize) -> Self {
        PyHce {
            inner: self
                .inner
                .clone()
                .with_chunk_spec(ChunkSpec::fixed(char_size)),
        }
    }

    fn with_chunk_pattern(&self, pattern: Vec<usize>) -> Self {
        PyHce {
            inner: self
                .inner
                .clone()
                .with_chunk_spec(ChunkSpec::pattern(&pattern)),
        }
    }

    #[pyo3(signature = (epoch_ms, granularity = "month"))]
    fn with_timestamp_config(&self, epoch_ms: i64, granularity: &str) -> PyResult<Self> {
        let g = match granularity {
            "second" => TsGranularity::Second,
            "minute" => TsGranularity::Minute,
            "hour" => TsGranularity::Hour,
            "day" => TsGranularity::Day,
            "week" => TsGranularity::Week,
            "month" => TsGranularity::Month,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "granularity must be one of: second, minute, hour, day, week, month",
                ))
            }
        };
        Ok(PyHce {
            inner: self.inner.clone().with_timestamp_config(epoch_ms, g),
        })
    }

    fn encode(&self, data: &[u8]) -> String {
        self.inner.encode(data)
    }

    fn decode(&self, input: &str) -> PyResult<Vec<u8>> {
        self.inner
            .decode(input)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn recover(&self, input: &str) -> PyResult<Vec<u8>> {
        let result = self
            .inner
            .recover(input)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        match result {
            RecoveryResult::Ok | RecoveryResult::Corrected(_) => self
                .inner
                .decode_corrected(&result)
                .or_else(|| self.inner.decode(input).ok())
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        "failed to decode after recovery",
                    )
                }),
            RecoveryResult::Ambiguous(n) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("ambiguous: {} candidates", n),
            )),
            RecoveryResult::Reject => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "reject: unrecoverable",
            )),
        }
    }
}

#[pymodule]
fn hce(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHce>()?;
    Ok(())
}
