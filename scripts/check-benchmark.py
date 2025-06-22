#!/usr/bin/env python3
"""
Benchmark Performance Check Script
Check if benchmark results exceed baseline thresholds
"""

import json
import sys
import os
from pathlib import Path

def load_json_file(file_path):
    """Load JSON file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"❌ File not found: {file_path}")
        return None
    except json.JSONDecodeError as e:
        print(f"❌ JSON parse error: {e}")
        return None

def check_benchmark_performance(results_file, baseline_file):
    """Check benchmark performance against baseline"""
    print("🔍 Benchmark Performance Check")
    print("=" * 50)
    
    # 加载结果文件
    results = load_json_file(results_file)
    if not results:
        return False
    
    # 加载基线文件
    baseline_data = load_json_file(baseline_file)
    if not baseline_data:
        print("⚠️  No baseline file found, skipping check")
        return True
    
    baseline = baseline_data.get('baselines', {})
    
    failed_tests = 0
    warning_tests = 0
    passed_tests = 0
    total_tests = len(results)
    
    for result in results:
        operation = result.get('operation', 'unknown')
        tool = result.get('tool', 'unknown')
        duration_ms = result.get('duration_ms', 0)
        success = result.get('success', False)
        
        key = f"{operation}_{tool}"
        
        if not success:
            print(f"❌ {tool} {operation}: FAILED")
            failed_tests += 1
            continue
        
        if key in baseline:
            baseline_op = baseline[key]
            max_duration = baseline_op.get('max_duration_ms', float('inf'))
            percentile_95 = baseline_op.get('percentile_95_ms', float('inf'))
            
            if duration_ms > max_duration:
                deviation = ((duration_ms - max_duration) / max_duration) * 100
                print(f"❌ {tool} {operation}: {duration_ms}ms > {max_duration}ms (+{deviation:.1f}%)")
                failed_tests += 1
            elif duration_ms > percentile_95:
                deviation = ((duration_ms - percentile_95) / percentile_95) * 100
                print(f"⚠️  {tool} {operation}: {duration_ms}ms > {percentile_95}ms (+{deviation:.1f}%)")
                warning_tests += 1
            else:
                print(f"✅ {tool} {operation}: {duration_ms}ms (within baseline)")
                passed_tests += 1
        else:
            print(f"❓ {tool} {operation}: {duration_ms}ms (no baseline)")
            warning_tests += 1
    
    print("=" * 50)
    print(f"Total: {total_tests}, Passed: {passed_tests}, Failed: {failed_tests}, Warnings: {warning_tests}")
    
    if failed_tests > 0:
        print("❌ BENCHMARK CHECK FAILED: Performance regression detected")
        return False
    elif warning_tests > 0:
        print("⚠️  BENCHMARK CHECK WARNING: Some tests have warnings")
        return True
    else:
        print("✅ BENCHMARK CHECK PASSED: All tests within baseline")
        return True

def find_latest_benchmark_file(results_dir):
    """Find the latest benchmark results file"""
    results_path = Path(results_dir)
    if not results_path.exists():
        return None
    
    benchmark_files = list(results_path.glob("benchmark_results_*.json"))
    if not benchmark_files:
        return None
    
    # Sort by modification time, return the latest
    latest_file = max(benchmark_files, key=lambda f: f.stat().st_mtime)
    return str(latest_file)

def main():
    """Main function"""
    if len(sys.argv) > 1:
        results_file = sys.argv[1]
    else:
        # Auto-find the latest benchmark results file
        results_file = find_latest_benchmark_file("benchmarks/results")
        if not results_file:
            print("❌ No benchmark results file found")
            print("Usage: python check-benchmark.py [results_file.json]")
            sys.exit(1)
        print(f"📊 Using latest benchmark results: {results_file}")
    
    baseline_file = "benchmarks/baseline.json"
    
    success = check_benchmark_performance(results_file, baseline_file)
    
    if not success:
        sys.exit(1)

if __name__ == "__main__":
    main()
