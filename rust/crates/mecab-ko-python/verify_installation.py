#!/usr/bin/env python3
"""
Verification script for mecab-ko Python bindings

Run this script after installation to verify everything works correctly.
"""

import sys


def check_import():
    """Check if the module can be imported"""
    print("=" * 60)
    print("Step 1: Checking module import...")
    print("=" * 60)
    try:
        import mecab_ko
        print("✅ mecab_ko module imported successfully")
        return True
    except ImportError as e:
        print(f"❌ Failed to import mecab_ko: {e}")
        print("\nPlease install the module first:")
        print("  cd /home/mare/mecab-ko/rust/crates/mecab-ko-python")
        print("  maturin develop --release")
        return False


def check_class():
    """Check if Mecab class can be instantiated"""
    print("\n" + "=" * 60)
    print("Step 2: Checking Mecab class...")
    print("=" * 60)
    try:
        from mecab_ko import Mecab
        mecab = Mecab()
        print(f"✅ Mecab instance created: {mecab}")
        return True, mecab
    except Exception as e:
        print(f"❌ Failed to create Mecab instance: {e}")
        return False, None


def check_methods(mecab):
    """Check if all methods work"""
    print("\n" + "=" * 60)
    print("Step 3: Checking API methods...")
    print("=" * 60)

    test_text = "안녕하세요"
    results = {}

    # Test morphs
    try:
        result = mecab.morphs(test_text)
        print(f"✅ morphs('{test_text}'): {result}")
        results['morphs'] = True
    except Exception as e:
        print(f"❌ morphs() failed: {e}")
        results['morphs'] = False

    # Test nouns
    try:
        result = mecab.nouns(test_text)
        print(f"✅ nouns('{test_text}'): {result}")
        results['nouns'] = True
    except Exception as e:
        print(f"❌ nouns() failed: {e}")
        results['nouns'] = False

    # Test pos
    try:
        result = mecab.pos(test_text)
        print(f"✅ pos('{test_text}'): {result}")
        results['pos'] = True
    except Exception as e:
        print(f"❌ pos() failed: {e}")
        results['pos'] = False

    # Test parse
    try:
        result = mecab.parse(test_text)
        print(f"✅ parse('{test_text}'):")
        for line in result.split('\n')[:3]:  # Show first 3 lines
            print(f"   {line}")
        results['parse'] = True
    except Exception as e:
        print(f"❌ parse() failed: {e}")
        results['parse'] = False

    # Test wakati
    try:
        result = mecab.wakati(test_text)
        print(f"✅ wakati('{test_text}'): {result}")
        results['wakati'] = True
    except Exception as e:
        print(f"❌ wakati() failed: {e}")
        results['wakati'] = False

    return all(results.values())


def check_metadata():
    """Check module metadata"""
    print("\n" + "=" * 60)
    print("Step 4: Checking module metadata...")
    print("=" * 60)
    try:
        import mecab_ko
        version = getattr(mecab_ko, '__version__', 'unknown')
        doc = getattr(mecab_ko, '__doc__', 'No documentation')
        print(f"✅ Module version: {version}")
        print(f"✅ Module doc: {doc}")
        return True
    except Exception as e:
        print(f"❌ Failed to check metadata: {e}")
        return False


def check_edge_cases(mecab):
    """Check edge cases"""
    print("\n" + "=" * 60)
    print("Step 5: Checking edge cases...")
    print("=" * 60)

    test_cases = [
        ("Empty string", ""),
        ("English text", "Hello World"),
        ("Mixed text", "Hello 안녕 World"),
        ("Numbers", "12345"),
        ("Special chars", "!@#$%"),
    ]

    all_passed = True
    for name, text in test_cases:
        try:
            result = mecab.morphs(text)
            print(f"✅ {name}: morphs('{text}') = {result}")
        except Exception as e:
            print(f"❌ {name} failed: {e}")
            all_passed = False

    return all_passed


def print_summary(results):
    """Print summary of all checks"""
    print("\n" + "=" * 60)
    print("VERIFICATION SUMMARY")
    print("=" * 60)

    total = len(results)
    passed = sum(results.values())

    for check, status in results.items():
        status_icon = "✅" if status else "❌"
        print(f"{status_icon} {check}")

    print(f"\nTotal: {passed}/{total} checks passed")

    if passed == total:
        print("\n🎉 All checks passed! The installation is working correctly.")
        return 0
    else:
        print(f"\n⚠️  {total - passed} check(s) failed. Please review the errors above.")
        return 1


def main():
    """Run all verification checks"""
    print("MeCab-Ko Python Bindings Verification")
    print("=" * 60)

    results = {}

    # Check 1: Import
    if not check_import():
        return 1

    # Check 2: Class instantiation
    success, mecab = check_class()
    results['Module Import'] = success
    if not success:
        return print_summary(results)

    # Check 3: Methods
    results['API Methods'] = check_methods(mecab)

    # Check 4: Metadata
    results['Module Metadata'] = check_metadata()

    # Check 5: Edge cases
    results['Edge Cases'] = check_edge_cases(mecab)

    # Summary
    return print_summary(results)


if __name__ == "__main__":
    sys.exit(main())
