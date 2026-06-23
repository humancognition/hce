# HCE — SQL (MySQL / MariaDB)

## Install

```bash
cc -shared -fPIC -I $(mysql_config --include) -I ../../crates/hce-ffi \
   -o libhce_udf.so hce_udf.c -L ../../target/release -lhce_ffi
cp libhce_udf.so $(mysql_config --plugindir)/
```

```sql
CREATE FUNCTION hce_encodeode RETURNS STRING SONAME 'libhce_udf.so';
CREATE FUNCTION hce_decodeode RETURNS STRING SONAME 'libhce_udf.so';
```

## Usage

```sql
SELECT hce_encodeode(UNHEX('0195e3a07c2e7b418f3d9a6c1e0b4d27'));
SELECT HEX(hce_decodeode('PETREN-NISLORPEN-LAFLER-SRORGULGOLFUN-PREPLEN'));

SELECT hce_encodeode(data_col, 'custom-32-byte-key-here-xxxxx!!', 0, 0) FROM items;
SELECT HEX(hce_decodeode(hce_col, 'custom-32-byte-key-here-xxxxx!!', 0, 0)) FROM items;
```

## API

| Function | Args | Returns |
|----------|------|---------|
| `hce_encodeode(data, key?, level?, mode?)` | `data` BLOB, `key` VARCHAR, `level` INT, `mode` INT | VARCHAR |
| `hce_decodeode(hce_str, key?, level?, mode?)` | `hce_str` VARCHAR, `key` VARCHAR, `level` INT, `mode` INT | BINARY(16) |

Level: 0=universal, 1=eu, 2=en, 3=numeric
Mode: 0=sealed, 1=open, 2=plain

## License

MIT
