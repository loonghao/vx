#!/usr/bin/env python3
"""
VX Benchmark Management Tool

This script helps manage benchmark results, generate reports, and track performance over time.
"""

import json
import os
import sys
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional
import statistics

class BenchmarkManager:
    def __init__(self, benchmarks_dir: Path):
        self.benchmarks_dir = benchmarks_dir
        self.results_dir = benchmarks_dir / "results"
        self.baseline_file = benchmarks_dir / "baseline.json"
        
        # Ensure directories exist
        self.results_dir.mkdir(exist_ok=True)
    
    def list_results(self) -> List[Path]:
        """List all benchmark result files."""
        return sorted(self.results_dir.glob("benchmark_results_*.json"))
    
    def load_result(self, file_path: Path) -> List[Dict[str, Any]]:
        """Load a benchmark result file."""
        try:
            # Try utf-8-sig first to handle BOM, fallback to utf-8
            for encoding in ['utf-8-sig', 'utf-8']:
                try:
                    with open(file_path, 'r', encoding=encoding) as f:
                        return json.load(f)
                except UnicodeDecodeError:
                    continue
            # If both encodings fail, raise the last exception
            with open(file_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            print(f"Error loading {file_path}: {e}")
            return []
    
    def load_baseline(self) -> Optional[Dict[str, Any]]:
        """Load baseline benchmark data."""
        if self.baseline_file.exists():
            try:
                # Try utf-8-sig first to handle BOM, fallback to utf-8
                for encoding in ['utf-8-sig', 'utf-8']:
                    try:
                        with open(self.baseline_file, 'r', encoding=encoding) as f:
                            return json.load(f)
                    except UnicodeDecodeError:
                        continue
                # If both encodings fail, raise the last exception
                with open(self.baseline_file, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except Exception as e:
                print(f"Error loading baseline: {e}")
        return None
    
    def save_baseline(self, data: Dict[str, Any]):
        """Save baseline benchmark data."""
        with open(self.baseline_file, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
    
    def generate_summary(self, result_file: Optional[Path] = None) -> Dict[str, Any]:
        """Generate a summary of benchmark results."""
        if result_file:
            files = [result_file]
        else:
            files = self.list_results()
        
        if not files:
            return {"error": "No benchmark results found"}
        
        # Use the latest result file if none specified
        latest_file = files[-1] if not result_file else result_file
        results = self.load_result(latest_file)
        
        if not results:
            return {"error": f"No data in {latest_file}"}
        
        summary = {
            "file": latest_file.name,
            "timestamp": results[0].get("timestamp") if results else None,
            "total_operations": len(results),
            "operations": {},
            "tools": {},
            "performance": {}
        }
        
        # Group by operation type
        for result in results:
            op_type = result.get("operation", "unknown")
            tool = result.get("tool", "unknown")
            duration = result.get("duration_ms", 0)
            success = result.get("success", False)
            
            # Operation statistics
            if op_type not in summary["operations"]:
                summary["operations"][op_type] = {
                    "count": 0,
                    "success_count": 0,
                    "total_duration": 0,
                    "durations": []
                }
            
            summary["operations"][op_type]["count"] += 1
            summary["operations"][op_type]["total_duration"] += duration
            summary["operations"][op_type]["durations"].append(duration)
            if success:
                summary["operations"][op_type]["success_count"] += 1
            
            # Tool statistics
            if tool not in summary["tools"]:
                summary["tools"][tool] = {
                    "operations": {},
                    "total_duration": 0,
                    "success_count": 0,
                    "total_count": 0
                }
            
            if op_type not in summary["tools"][tool]["operations"]:
                summary["tools"][tool]["operations"][op_type] = {
                    "count": 0,
                    "duration": 0,
                    "success": 0
                }
            
            summary["tools"][tool]["operations"][op_type]["count"] += 1
            summary["tools"][tool]["operations"][op_type]["duration"] += duration
            if success:
                summary["tools"][tool]["operations"][op_type]["success"] += 1
            
            summary["tools"][tool]["total_duration"] += duration
            summary["tools"][tool]["total_count"] += 1
            if success:
                summary["tools"][tool]["success_count"] += 1
        
        # Calculate averages and statistics
        for op_type, data in summary["operations"].items():
            if data["durations"]:
                data["avg_duration"] = statistics.mean(data["durations"])
                data["median_duration"] = statistics.median(data["durations"])
                data["min_duration"] = min(data["durations"])
                data["max_duration"] = max(data["durations"])
                if len(data["durations"]) > 1:
                    data["std_duration"] = statistics.stdev(data["durations"])
                else:
                    data["std_duration"] = 0
            data["success_rate"] = data["success_count"] / data["count"] if data["count"] > 0 else 0
            del data["durations"]  # Remove raw data to keep summary clean
        
        return summary
    
    def compare_with_baseline(self, result_file: Optional[Path] = None) -> Dict[str, Any]:
        """Compare current results with baseline."""
        baseline = self.load_baseline()
        if not baseline:
            return {"error": "No baseline found. Run 'set-baseline' first."}
        
        current_summary = self.generate_summary(result_file)
        if "error" in current_summary:
            return current_summary
        
        comparison = {
            "baseline_file": baseline.get("source_file", "unknown"),
            "current_file": current_summary["file"],
            "improvements": [],
            "regressions": [],
            "new_operations": [],
            "missing_operations": []
        }
        
        baseline_ops = baseline.get("operations", {})
        current_ops = current_summary.get("operations", {})
        
        # Compare operations
        for op_type in set(baseline_ops.keys()) | set(current_ops.keys()):
            if op_type not in baseline_ops:
                comparison["new_operations"].append(op_type)
                continue
            elif op_type not in current_ops:
                comparison["missing_operations"].append(op_type)
                continue
            
            baseline_avg = baseline_ops[op_type].get("avg_duration", 0)
            current_avg = current_ops[op_type].get("avg_duration", 0)
            
            if baseline_avg > 0:
                change_percent = ((current_avg - baseline_avg) / baseline_avg) * 100
                
                change_data = {
                    "operation": op_type,
                    "baseline_avg": baseline_avg,
                    "current_avg": current_avg,
                    "change_percent": change_percent,
                    "change_ms": current_avg - baseline_avg
                }
                
                if change_percent < -5:  # Improvement > 5%
                    comparison["improvements"].append(change_data)
                elif change_percent > 5:  # Regression > 5%
                    comparison["regressions"].append(change_data)
        
        return comparison
    
    def set_baseline(self, result_file: Optional[Path] = None):
        """Set a benchmark result as the baseline."""
        if result_file and not result_file.exists():
            print(f"Error: File {result_file} does not exist")
            return False
        
        summary = self.generate_summary(result_file)
        if "error" in summary:
            print(f"Error generating summary: {summary['error']}")
            return False
        
        baseline_data = {
            "created_at": datetime.now().isoformat(),
            "source_file": summary["file"],
            "operations": summary["operations"],
            "tools": summary["tools"]
        }
        
        self.save_baseline(baseline_data)
        print(f"✅ Baseline set from {summary['file']}")
        return True
    
    def generate_report(self, output_file: Optional[Path] = None) -> str:
        """Generate a detailed performance report."""
        files = self.list_results()
        if not files:
            return "No benchmark results found."
        
        latest_file = files[-1]
        summary = self.generate_summary(latest_file)
        comparison = self.compare_with_baseline(latest_file)
        
        report_lines = [
            "# VX Performance Report",
            f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"Latest Results: {summary['file']}",
            "",
            "## Summary",
            f"- Total Operations: {summary['total_operations']}",
            f"- Timestamp: {summary.get('timestamp', 'N/A')}",
            ""
        ]
        
        # Operations summary
        report_lines.extend([
            "## Operations Performance",
            "| Operation | Count | Success Rate | Avg Duration (ms) | Min | Max | Median |",
            "|-----------|-------|--------------|-------------------|-----|-----|--------|"
        ])
        
        for op_type, data in summary["operations"].items():
            success_rate = f"{data['success_rate']:.1%}"
            avg_dur = f"{data['avg_duration']:.1f}"
            min_dur = f"{data['min_duration']:.1f}"
            max_dur = f"{data['max_duration']:.1f}"
            median_dur = f"{data['median_duration']:.1f}"
            
            report_lines.append(
                f"| {op_type} | {data['count']} | {success_rate} | {avg_dur} | {min_dur} | {max_dur} | {median_dur} |"
            )
        
        report_lines.append("")
        
        # Tools summary
        report_lines.extend([
            "## Tools Performance",
            "| Tool | Operations | Success Rate | Total Duration (ms) |",
            "|------|------------|--------------|---------------------|"
        ])
        
        for tool, data in summary["tools"].items():
            success_rate = f"{data['success_count'] / data['total_count']:.1%}" if data['total_count'] > 0 else "0%"
            report_lines.append(
                f"| {tool} | {data['total_count']} | {success_rate} | {data['total_duration']:.1f} |"
            )
        
        report_lines.append("")
        
        # Comparison with baseline
        if "error" not in comparison:
            report_lines.extend([
                "## Comparison with Baseline",
                f"Baseline: {comparison['baseline_file']}",
                ""
            ])
            
            if comparison["improvements"]:
                report_lines.extend([
                    "### 🚀 Improvements",
                    "| Operation | Baseline (ms) | Current (ms) | Improvement |",
                    "|-----------|---------------|--------------|-------------|"
                ])
                for imp in comparison["improvements"]:
                    change = f"{abs(imp['change_percent']):.1f}% ({abs(imp['change_ms']):.1f}ms faster)"
                    report_lines.append(
                        f"| {imp['operation']} | {imp['baseline_avg']:.1f} | {imp['current_avg']:.1f} | {change} |"
                    )
                report_lines.append("")
            
            if comparison["regressions"]:
                report_lines.extend([
                    "### ⚠️ Regressions",
                    "| Operation | Baseline (ms) | Current (ms) | Regression |",
                    "|-----------|---------------|--------------|------------|"
                ])
                for reg in comparison["regressions"]:
                    change = f"{reg['change_percent']:.1f}% ({reg['change_ms']:.1f}ms slower)"
                    report_lines.append(
                        f"| {reg['operation']} | {reg['baseline_avg']:.1f} | {reg['current_avg']:.1f} | {change} |"
                    )
                report_lines.append("")
            
            if comparison["new_operations"]:
                report_lines.extend([
                    "### ✨ New Operations",
                    ", ".join(comparison["new_operations"]),
                    ""
                ])
            
            if comparison["missing_operations"]:
                report_lines.extend([
                    "### ❌ Missing Operations",
                    ", ".join(comparison["missing_operations"]),
                    ""
                ])
        else:
            report_lines.extend([
                "## Baseline Comparison",
                f"⚠️ {comparison['error']}",
                ""
            ])
        
        report_content = "\n".join(report_lines)
        
        if output_file:
            with open(output_file, 'w', encoding='utf-8') as f:
                f.write(report_content)
            print(f"📊 Report saved to {output_file}")
        
        return report_content

def main():
    parser = argparse.ArgumentParser(description="VX Benchmark Management Tool")
    parser.add_argument("--benchmarks-dir", type=Path, default=Path(__file__).parent,
                       help="Path to benchmarks directory")
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # List command
    list_parser = subparsers.add_parser("list", help="List all benchmark results")
    
    # Summary command
    summary_parser = subparsers.add_parser("summary", help="Generate summary of latest results")
    summary_parser.add_argument("--file", type=Path, help="Specific result file to summarize")
    
    # Compare command
    compare_parser = subparsers.add_parser("compare", help="Compare with baseline")
    compare_parser.add_argument("--file", type=Path, help="Specific result file to compare")
    
    # Set baseline command
    baseline_parser = subparsers.add_parser("set-baseline", help="Set baseline from results")
    baseline_parser.add_argument("--file", type=Path, help="Specific result file to use as baseline")
    
    # Report command
    report_parser = subparsers.add_parser("report", help="Generate detailed report")
    report_parser.add_argument("--output", type=Path, help="Output file for report")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    manager = BenchmarkManager(args.benchmarks_dir)
    
    if args.command == "list":
        files = manager.list_results()
        if files:
            print("📊 Available benchmark results:")
            for file in files:
                print(f"  - {file.name}")
        else:
            print("No benchmark results found.")
    
    elif args.command == "summary":
        summary = manager.generate_summary(args.file)
        if "error" in summary:
            print(f"❌ {summary['error']}")
        else:
            print(f"📊 Summary for {summary['file']}:")
            print(f"  Total Operations: {summary['total_operations']}")
            print(f"  Timestamp: {summary.get('timestamp', 'N/A')}")
            print("\n  Operations:")
            for op_type, data in summary["operations"].items():
                print(f"    {op_type}: {data['count']} ops, {data['success_rate']:.1%} success, {data['avg_duration']:.1f}ms avg")
    
    elif args.command == "compare":
        comparison = manager.compare_with_baseline(args.file)
        if "error" in comparison:
            print(f"❌ {comparison['error']}")
        else:
            print(f"📊 Comparison: {comparison['current_file']} vs {comparison['baseline_file']}")
            if comparison["improvements"]:
                print(f"  🚀 {len(comparison['improvements'])} improvements")
            if comparison["regressions"]:
                print(f"  ⚠️ {len(comparison['regressions'])} regressions")
            if comparison["new_operations"]:
                print(f"  ✨ {len(comparison['new_operations'])} new operations")
    
    elif args.command == "set-baseline":
        manager.set_baseline(args.file)
    
    elif args.command == "report":
        report = manager.generate_report(args.output)
        if not args.output:
            print(report)

if __name__ == "__main__":
    main()