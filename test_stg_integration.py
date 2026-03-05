#!/usr/bin/env python3
"""
Integration test for nautilus-stg via nautilus_pyo3.

This test verifies that the STG module is properly integrated into
the NautilusTrader Python package and all functionality works.
"""

from nautilus_trader.core import nautilus_pyo3


def test_imports():
    """Test that all STG classes can be imported."""
    assert hasattr(nautilus_pyo3, "stg"), "stg module not found in nautilus_pyo3"
    
    stg = nautilus_pyo3.stg
    
    # Verify all expected classes are available
    expected_classes = [
        "LocalL2Book",
        "Side",
        "ImpactResult",
        "LiquidityStats",
        "MarketState",
        "CostEstimatorConfig",
    ]
    
    for cls_name in expected_classes:
        assert hasattr(stg, cls_name), f"{cls_name} not found in stg module"
    
    print("✓ All STG classes are available")


def test_basic_functionality():
    """Test basic LocalL2Book functionality."""
    stg = nautilus_pyo3.stg
    
    # Create a book
    book = stg.LocalL2Book(top_k=50)
    
    # Update with some data
    book.update(
        side=stg.Side.buy(),
        price=100.0,
        size=10.0,
    )
    book.update(
        side=stg.Side.sell(),
        price=100.10,
        size=15.0,
    )
    
    # Check best prices
    stats = book.best()
    assert stats.bid1 == 100.0, f"Expected bid1 100.0, got {stats.bid1}"
    assert stats.ask1 == 100.10, f"Expected ask1 100.10, got {stats.ask1}"
    assert abs(stats.mid - 100.05) < 0.001, f"Expected mid 100.05, got {stats.mid}"
    
    print("✓ Basic functionality works")


def test_microprice():
    """Test microprice calculation."""
    stg = nautilus_pyo3.stg
    
    book = stg.LocalL2Book(top_k=50)
    
    # Add bid and ask
    book.update(stg.Side.buy(), 100.0, 10.0)
    book.update(stg.Side.sell(), 100.10, 20.0)
    
    # Calculate microprice
    mp = book.microprice()
    assert mp is not None, "Microprice should not be None"
    
    # Expected: (100 * 20 + 100.10 * 10) / (10 + 20) = 100.0333...
    expected = (100.0 * 20.0 + 100.10 * 10.0) / 30.0
    assert abs(mp - expected) < 0.0001, f"Expected microprice {expected}, got {mp}"
    
    # Test deviation
    dev = book.microprice_deviation_bps()
    assert dev is not None, "Microprice deviation should not be None"
    
    print(f"✓ Microprice calculation works: {mp:.4f} (deviation: {dev:.2f} bps)")


def test_impact_calculation():
    """Test impact price calculation."""
    stg = nautilus_pyo3.stg
    
    book = stg.LocalL2Book(top_k=50)
    
    # Build a simple book
    book.update(stg.Side.buy(), 100.0, 10.0)
    book.update(stg.Side.buy(), 99.9, 20.0)
    book.update(stg.Side.sell(), 100.10, 15.0)
    book.update(stg.Side.sell(), 100.20, 25.0)
    
    # Calculate impact for buying 12 units (should eat through first ask level)
    result = book.impact(side=stg.Side.buy(), qty=12.0, cushion_ticks=0)
    
    assert result.vwap_price >= 100.10, "VWAP for buy should be >= first ask"
    assert result.ok, "Should have enough liquidity"
    assert result.slip_bps >= 0, "Slippage should be non-negative"
    
    print(f"✓ Impact calculation works: VWAP={result.vwap_price:.4f}, slippage={result.slip_bps:.2f} bps")


def test_cost_estimation():
    """Test dynamic cost estimation."""
    stg = nautilus_pyo3.stg
    
    book = stg.LocalL2Book(top_k=50)
    
    # Build a book
    book.update(stg.Side.buy(), 100.0, 50.0)
    book.update(stg.Side.sell(), 100.10, 50.0)
    
    # Create market state (using actual parameter names from inspection)
    market_state = stg.MarketState(
        rvol_bps_1s=50.0,
        bc=0.9,
        lcd=False,
        withdraw_score=0.0,
        rate_limit_usage=0.1,
        latency_p95_ms=10.0,
    )
    
    # Estimate bailout cost for 30 units (selling)
    cost_bps = book.bailout_cost_bps_est(qty=30.0, side=stg.Side.sell(), market_state=market_state)
    
    assert cost_bps > 0, "Cost should be positive"
    assert cost_bps < 1000, "Cost should be reasonable"
    
    print(f"✓ Cost estimation works: {cost_bps:.2f} bps")


def test_custom_config():
    """Test custom cost estimator configuration."""
    stg = nautilus_pyo3.stg
    
    # Create custom config with Python API parameter names
    config = stg.CostEstimatorConfig(
        fee_taker_bps=5.0,
        expected_exec_time_ms=100.0,
        drift_rate_per_rvol=0.5,
        bc_penalty_factor=20.0,
        lcd_penalty_bps=50.0,
        withdraw_penalty_factor=10.0,
        rate_limit_penalty_factor=30.0,
        latency_drift_rate_per_ms=0.1,
        insufficient_liquidity_penalty_bps=50.0,
        extrapolation_ratio_limit=3.0,
        no_liquidity_penalty_bps=200.0,
    )
    
    book = stg.LocalL2Book(top_k=50, top_n=10)
    book.set_cost_config(config)
    
    # Verify config was set
    retrieved_config = book.cost_config()
    assert abs(retrieved_config.fee_taker_bps - 5.0) < 0.001
    
    print("✓ Custom configuration works")


def test_top_k_functionality():
    """Test top_k retrieval."""
    stg = nautilus_pyo3.stg
    
    book = stg.LocalL2Book(top_k=3)  # Only keep top 3 levels
    
    # Add more than 3 levels
    for i in range(5):
        price = 100.0 - i * 0.1
        book.update(stg.Side.buy(), price, 10.0)
    
    # Get top 2 levels from buy side
    prices, sizes = book.top_k(side=stg.Side.buy(), k=2)
    assert len(prices) <= 2, f"Should return at most 2 levels, got {len(prices)}"
    assert len(sizes) == len(prices), "prices and sizes should have same length"
    
    print(f"✓ top_k retrieval works: got {len(prices)} levels")


def run_all_tests():
    """Run all integration tests."""
    print("=" * 60)
    print("STG Integration Tests via nautilus_pyo3")
    print("=" * 60)
    print()
    
    tests = [
        ("Import Test", test_imports),
        ("Basic Functionality", test_basic_functionality),
        ("Microprice", test_microprice),
        ("Impact Calculation", test_impact_calculation),
        ("Cost Estimation", test_cost_estimation),
        ("Custom Config", test_custom_config),
        ("Top-K Limiting", test_top_k_functionality),
    ]
    
    passed = 0
    failed = 0
    
    for name, test_func in tests:
        try:
            print(f"Running: {name}...")
            test_func()
            passed += 1
        except Exception as e:
            print(f"✗ {name} failed: {e}")
            failed += 1
        print()
    
    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    print("=" * 60)
    
    if failed > 0:
        exit(1)


if __name__ == "__main__":
    run_all_tests()
