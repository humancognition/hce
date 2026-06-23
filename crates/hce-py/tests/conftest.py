import pytest


@pytest.fixture(scope="session")
def hce_module():
    import hce
    return hce


KEY = b"py-test-key-32-bytes-long-key!!"
UUID = bytes.fromhex("0195e3a07c2e7b418f3d9a6c1e0b4d27")


@pytest.fixture
def sealed(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="sealed")


@pytest.fixture
def sealed_lower(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="sealed").with_case("lower")


@pytest.fixture
def sealed_cs3(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="sealed").with_check_syllables(3)


@pytest.fixture
def sealed_none(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="sealed").with_chunk_none()


@pytest.fixture
def sealed_fixed(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="sealed").with_chunk_fixed(7)


@pytest.fixture
def sealed_pattern(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="sealed").with_chunk_pattern([4, 4, 4, 2])


@pytest.fixture
def sealed_dot(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="sealed").with_separator(".")


@pytest.fixture
def plain(hce_module):
    return hce_module.Hce(key=None, level="universal", mode="plain")


@pytest.fixture
def open_hce(hce_module):
    return hce_module.Hce(key=KEY, level="universal", mode="open")


@pytest.fixture
def open_ts(hce_module):
    return (hce_module.Hce(key=KEY, level="universal", mode="open")
            .with_timestamp_config(1_800_000_000_000, "month"))
