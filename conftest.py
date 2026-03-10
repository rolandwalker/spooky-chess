import pytest


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption("--run-slow", action="store_true", default=False, help="run slow tests")


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line("markers", "slow: mark test as slow to run, meaning it will be skipped by default")


def pytest_collection_modifyitems(config: pytest.Config, items: list[pytest.Item]) -> None:
    if config.getoption("--run-slow") is False:
        # skip @pytest.mark.slow functions by default:
        skip_reason = pytest.mark.skip(reason="need --run-slow command-line option to run")

        for item in items:
            if "slow" in item.keywords:
                item.add_marker(skip_reason)
