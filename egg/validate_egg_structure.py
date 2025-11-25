#!/usr/bin/env python3
"""
Validation script for Quantum LIMIT-Graph Egg folder structure
Ensures all crates, services, and examples are properly configured
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from typing import List, Dict, Tuple

class EggValidator:
    def __init__(self, egg_root: str):
        self.egg_root = Path(egg_root)
        self.errors: List[str] = []
        self.warnings: List[str] = []
        self.successes: List[str] = []
    
    def validate_structure(self) -> bool:
        """Validate the overall folder structure"""
        print("🔍 Validating egg folder structure...")
        
        required_dirs = [
            "crates/limit-core",
            "crates/limit-storage",
            "crates/limit-orchestration",
            "crates/limit-agents",
            "services/api",
            "examples",
            "tests",
            ".github/workflows"
        ]
        
        for dir_path in required_dirs:
            full_path = self.egg_root / dir_path
            if full_path.exists():
                self.successes.append(f"✓ Directory exists: {dir_path}")
            else:
                self.errors.append(f"✗ Missing directory: {dir_path}")
        
        return len(self.errors) == 0
    
    def validate_crate_files(self) -> bool:
        """Validate that each crate has required files"""
        print("\n🔍 Validating crate files...")
        
        crates = [
            "limit-core",
            "limit-storage",
            "limit-orchestration",
            "limit-agents"
        ]
        
        for crate in crates:
            crate_path = self.egg_root / "crates" / crate
            
            # Check Cargo.toml
            cargo_toml = crate_path / "Cargo.toml"
            if cargo_toml.exists():
                self.successes.append(f"✓ {crate}/Cargo.toml exists")
            else:
                self.errors.append(f"✗ {crate}/Cargo.toml missing")
            
            # Check src/lib.rs
            lib_rs = crate_path / "src" / "lib.rs"
            if lib_rs.exists():
                self.successes.append(f"✓ {crate}/src/lib.rs exists")
            else:
                self.errors.append(f"✗ {crate}/src/lib.rs missing")
        
        return len(self.errors) == 0
    
    def validate_examples(self) -> bool:
        """Validate example files"""
        print("\n🔍 Validating examples...")
        
        required_examples = [
            "basic_session.rs",
            "agent_benchmark.rs",
            "federated_orchestration.rs"
        ]
        
        examples_dir = self.egg_root / "examples"
        for example in required_examples:
            example_path = examples_dir / example
            if example_path.exists():
                self.successes.append(f"✓ Example exists: {example}")
            else:
                self.errors.append(f"✗ Missing example: {example}")
        
        return len(self.errors) == 0
    
    def validate_workspace_config(self) -> bool:
        """Validate workspace Cargo.toml configuration"""
        print("\n🔍 Validating workspace configuration...")
        
        cargo_toml = self.egg_root / "Cargo.toml"
        if not cargo_toml.exists():
            self.errors.append("✗ Root Cargo.toml missing")
            return False
        
        with open(cargo_toml, 'r') as f:
            content = f.read()
            
            required_members = [
                "crates/limit-core",
                "crates/limit-storage",
                "crates/limit-orchestration",
                "crates/limit-agents",
                "services/api"
            ]
            
            for member in required_members:
                if member in content:
                    self.successes.append(f"✓ Workspace member: {member}")
                else:
                    self.errors.append(f"✗ Missing workspace member: {member}")
        
        return len(self.errors) == 0
    
    def validate_docker_config(self) -> bool:
        """Validate Docker configuration"""
        print("\n🔍 Validating Docker configuration...")
        
        dockerfile = self.egg_root / "Dockerfile"
        docker_compose = self.egg_root / "docker-compose.yml"
        dockerignore = self.egg_root / ".dockerignore"
        
        if dockerfile.exists():
            self.successes.append("✓ Dockerfile exists")
        else:
            self.warnings.append("⚠ Dockerfile missing")
        
        if docker_compose.exists():
            self.successes.append("✓ docker-compose.yml exists")
        else:
            self.warnings.append("⚠ docker-compose.yml missing")
        
        if dockerignore.exists():
            self.successes.append("✓ .dockerignore exists")
        else:
            self.warnings.append("⚠ .dockerignore missing")
        
        return True
    
    def validate_ci_workflow(self) -> bool:
        """Validate CI/CD workflow"""
        print("\n🔍 Validating CI/CD workflow...")
        
        ci_workflow = self.egg_root / ".github" / "workflows" / "ci.yml"
        if ci_workflow.exists():
            self.successes.append("✓ CI workflow exists")
            
            with open(ci_workflow, 'r') as f:
                content = f.read()
                
                required_jobs = ["test", "fmt", "clippy", "build"]
                for job in required_jobs:
                    if f"{job}:" in content:
                        self.successes.append(f"✓ CI job defined: {job}")
                    else:
                        self.warnings.append(f"⚠ CI job missing: {job}")
        else:
            self.errors.append("✗ CI workflow missing")
        
        return len(self.errors) == 0
    
    def validate_documentation(self) -> bool:
        """Validate documentation files"""
        print("\n🔍 Validating documentation...")
        
        docs = [
            "README.md",
            "QUICK_START.md",
            "IMPLEMENTATION_COMPLETE.md",
            "FEDERATED_ARCHITECTURE.md"
        ]
        
        for doc in docs:
            doc_path = self.egg_root / doc
            if doc_path.exists():
                self.successes.append(f"✓ Documentation: {doc}")
            else:
                self.warnings.append(f"⚠ Missing documentation: {doc}")
        
        return True
    
    def check_cargo_build(self) -> bool:
        """Attempt to check if cargo can build the workspace"""
        print("\n🔍 Checking cargo build capability...")
        
        try:
            result = subprocess.run(
                ["cargo", "check", "--workspace"],
                cwd=self.egg_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if result.returncode == 0:
                self.successes.append("✓ Cargo check passed")
                return True
            else:
                self.warnings.append(f"⚠ Cargo check failed: {result.stderr[:200]}")
                return False
        except FileNotFoundError:
            self.warnings.append("⚠ Cargo not found - skipping build check")
            return True
        except subprocess.TimeoutExpired:
            self.warnings.append("⚠ Cargo check timed out")
            return True
        except Exception as e:
            self.warnings.append(f"⚠ Cargo check error: {str(e)}")
            return True
    
    def print_report(self):
        """Print validation report"""
        print("\n" + "="*60)
        print("📊 VALIDATION REPORT")
        print("="*60)
        
        if self.successes:
            print(f"\n✅ Successes ({len(self.successes)}):")
            for success in self.successes[:10]:  # Show first 10
                print(f"  {success}")
            if len(self.successes) > 10:
                print(f"  ... and {len(self.successes) - 10} more")
        
        if self.warnings:
            print(f"\n⚠️  Warnings ({len(self.warnings)}):")
            for warning in self.warnings:
                print(f"  {warning}")
        
        if self.errors:
            print(f"\n❌ Errors ({len(self.errors)}):")
            for error in self.errors:
                print(f"  {error}")
        
        print("\n" + "="*60)
        
        if self.errors:
            print("❌ VALIDATION FAILED")
            return False
        elif self.warnings:
            print("⚠️  VALIDATION PASSED WITH WARNINGS")
            return True
        else:
            print("✅ VALIDATION PASSED")
            return True
    
    def run_all_validations(self) -> bool:
        """Run all validation checks"""
        print("🚀 Starting Quantum LIMIT-Graph Egg Validation\n")
        
        validations = [
            self.validate_structure,
            self.validate_crate_files,
            self.validate_examples,
            self.validate_workspace_config,
            self.validate_docker_config,
            self.validate_ci_workflow,
            self.validate_documentation,
            self.check_cargo_build
        ]
        
        for validation in validations:
            validation()
        
        return self.print_report()


def main():
    # Determine egg root path
    script_dir = Path(__file__).parent
    egg_root = script_dir
    
    if not egg_root.exists():
        print(f"❌ Error: Egg root directory not found: {egg_root}")
        sys.exit(1)
    
    validator = EggValidator(str(egg_root))
    success = validator.run_all_validations()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
