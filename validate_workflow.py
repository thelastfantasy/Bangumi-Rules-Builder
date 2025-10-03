#!/usr/bin/env python3
"""
GitHub Actions Workflow 验证脚本
用于在本地验证 workflow 配置的正确性
"""

import yaml
import os
import sys
from pathlib import Path

def validate_yaml_syntax(file_path):
    """验证 YAML 语法"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            yaml.safe_load(f)
        print("[OK] YAML 语法验证通过")
        return True
    except yaml.YAMLError as e:
        print(f"[ERROR] YAML 语法错误: {e}")
        return False

def validate_workflow_structure(file_path):
    """验证 workflow 结构"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            workflow = yaml.safe_load(f)

        errors = []

        # 检查必需字段
        required_fields = ['name']
        for field in required_fields:
            if field not in workflow:
                errors.append(f"缺少必需字段: {field}")

        # 检查触发条件 - 'on' 在 YAML 中会被解析为布尔值 True
        if True in workflow:  # 'on' 被解析为 True
            triggers = workflow[True]
            if isinstance(triggers, dict):
                if 'push' in triggers:
                    push_config = triggers['push']
                    if 'branches' in push_config:
                        branches = push_config['branches']
                        if 'main' not in branches:
                            errors.append("push 触发条件中缺少 'main' 分支")
                if 'workflow_dispatch' not in triggers:
                    print("[WARN] 建议添加 workflow_dispatch 触发条件")

        # 检查 jobs
        if 'jobs' not in workflow:
            errors.append("缺少 jobs 字段")
        else:
            jobs = workflow['jobs']
            if 'build' not in jobs:
                errors.append("缺少 'build' job")
            if 'create-release' not in jobs:
                errors.append("缺少 'create-release' job")

            # 检查 build job 结构
            if 'build' in jobs:
                build_job = jobs['build']
                if 'strategy' not in build_job:
                    errors.append("build job 缺少 strategy 配置")
                if 'steps' not in build_job:
                    errors.append("build job 缺少 steps 配置")

        if errors:
            print("[ERROR] Workflow 结构错误:")
            for error in errors:
                print(f"   - {error}")
            return False
        else:
            print("[OK] Workflow 结构验证通过")
            return True

    except Exception as e:
        print(f"[ERROR] Workflow 结构验证失败: {e}")
        return False

def validate_file_paths():
    """验证 workflow 中引用的文件路径"""
    workflow_dir = Path('.github/workflows')
    required_files = [
        'Cargo.toml',
        'src/main.rs',
        'qb_rule_editor.py',
        'run_editor.bat',
        'run_editor.ps1',
        'EDITOR_README.md',
        'tasks.json',
        'README.md'
    ]

    missing_files = []
    for file in required_files:
        if not Path(file).exists():
            missing_files.append(file)

    if missing_files:
        print("[ERROR] 缺少必需文件:")
        for file in missing_files:
            print(f"   - {file}")
        return False
    else:
        print("[OK] 所有必需文件都存在")
        return True

def main():
    """主函数"""
    workflow_file = '.github/workflows/release.yml'

    print("开始验证 GitHub Actions Workflow...")
    print("=" * 50)

    # 检查文件是否存在
    if not os.path.exists(workflow_file):
        print(f"❌ Workflow 文件不存在: {workflow_file}")
        sys.exit(1)

    print(f"验证文件: {workflow_file}")
    print()

    # 执行验证
    yaml_valid = validate_yaml_syntax(workflow_file)
    structure_valid = validate_workflow_structure(workflow_file)
    files_valid = validate_file_paths()

    print()
    print("=" * 50)

    # 总结
    if yaml_valid and structure_valid and files_valid:
        print("[SUCCESS] 所有验证通过！Workflow 配置正确。")
        print("\n建议下一步:")
        print("   1. 使用 'act' 工具进行本地测试")
        print("   2. 推送代码到 GitHub")
        print("   3. 在 main 分支创建标签触发发布")
        sys.exit(0)
    else:
        print("[ERROR] 验证失败，请修复上述问题")
        sys.exit(1)

if __name__ == "__main__":
    main()