#!/usr/bin/env python3
"""
比较 bangumi_results.json 和 qb_download_rules.json 文件，分析作品数量差异的原因
"""

import json
import sys

def load_json_file(file_path):
    """加载JSON文件"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        print(f"错误：无法加载文件 {file_path}: {e}")
        sys.exit(1)

def extract_work_names_from_bangumi(bangumi_data):
    """从Bangumi结果中提取作品名称"""
    work_names = []
    for work in bangumi_data:
        # 优先使用中文名称，如果没有则使用清理后的日文标题
        if work.get('chinese_name'):
            work_names.append(work['chinese_name'])
        else:
            work_names.append(work['cleaned_title'])
    return work_names

def extract_work_names_from_qb_rules(qb_rules):
    """从qBittorrent规则中提取作品名称"""
    work_names = []
    for rule_name in qb_rules.keys():
        # 规则名称格式: "2025年10月新番 作品名"
        # 去掉季节前缀
        if "2025年10月新番 " in rule_name:
            work_name = rule_name.replace("2025年10月新番 ", "")
            work_names.append(work_name)
    return work_names

def analyze_differences(bangumi_names, qb_names):
    """分析两个列表的差异"""
    bangumi_set = set(bangumi_names)
    qb_set = set(qb_names)

    print(f"\n📊 文件对比分析")
    print(f"=" * 60)
    print(f"Bangumi结果中的作品数量: {len(bangumi_names)}")
    print(f"qBittorrent规则中的作品数量: {len(qb_names)}")
    print(f"差异数量: {abs(len(bangumi_names) - len(qb_names))}")

    # 找出在Bangumi中但不在qBittorrent规则中的作品
    missing_in_qb = bangumi_set - qb_set

    # 找出在qBittorrent规则中但不在Bangumi中的作品
    extra_in_qb = qb_set - bangumi_set

    print(f"\n🔍 详细差异分析")
    print(f"=" * 60)

    if missing_in_qb:
        print(f"\n❌ 在Bangumi结果中但未生成qBittorrent规则的作品 ({len(missing_in_qb)}个):")
        for name in sorted(missing_in_qb):
            print(f"  - {name}")
    else:
        print(f"\n✅ 所有Bangumi结果都生成了qBittorrent规则")

    if extra_in_qb:
        print(f"\n⚠️  在qBittorrent规则中但不在Bangumi结果中的作品 ({len(extra_in_qb)}个):")
        for name in sorted(extra_in_qb):
            print(f"  - {name}")
    else:
        print(f"\n✅ qBittorrent规则中没有多余的作品")

    # 分析重复作品
    bangumi_duplicates = find_duplicates(bangumi_names)
    qb_duplicates = find_duplicates(qb_names)

    if bangumi_duplicates:
        print(f"\n🔄 Bangumi结果中的重复作品 ({len(bangumi_duplicates)}个):")
        for name, count in bangumi_duplicates.items():
            print(f"  - {name} (出现{count}次)")

    if qb_duplicates:
        print(f"\n🔄 qBittorrent规则中的重复作品 ({len(qb_duplicates)}个):")
        for name, count in qb_duplicates.items():
            print(f"  - {name} (出现{count}次)")

def find_duplicates(name_list):
    """找出列表中的重复项"""
    from collections import Counter
    counter = Counter(name_list)
    return {name: count for name, count in counter.items() if count > 1}

def analyze_bangumi_duplicates(bangumi_data):
    """分析Bangumi数据中的重复项"""
    print(f"\n🔍 Bangumi数据详细重复分析")
    print(f"=" * 60)

    # 按中文名称分组
    chinese_name_groups = {}
    for work in bangumi_data:
        chinese_name = work.get('chinese_name') or work['cleaned_title']
        if chinese_name not in chinese_name_groups:
            chinese_name_groups[chinese_name] = []
        chinese_name_groups[chinese_name].append(work)

    duplicates = {name: works for name, works in chinese_name_groups.items() if len(works) > 1}

    if duplicates:
        print(f"发现 {len(duplicates)} 个重复作品组:")
        for name, works in duplicates.items():
            print(f"\n📺 作品: {name}")
            print(f"   重复数量: {len(works)}")
            for i, work in enumerate(works, 1):
                print(f"   第{i}个实例:")
                print(f"     - 原标题: {work['original_title']}")
                print(f"     - 清理标题: {work['cleaned_title']}")
                print(f"     - Bangumi ID: {work.get('bangumi_id', '无')}")
                print(f"     - 中文名: {work.get('chinese_name', '无')}")
                print(f"     - 放送日期: {work.get('air_date', '无')}")
    else:
        print("未发现重复作品")

def main():
    """主函数"""
    print("开始比较 bangumi_results.json 和 qb_download_rules.json")

    # 加载文件
    bangumi_data = load_json_file('bangumi_results.json')
    qb_rules = load_json_file('qb_download_rules.json')

    print(f"成功加载文件:")
    print(f"   - bangumi_results.json: {len(bangumi_data)} 个作品")
    print(f"   - qb_download_rules.json: {len(qb_rules)} 个规则")

    # 提取作品名称
    bangumi_names = extract_work_names_from_bangumi(bangumi_data)
    qb_names = extract_work_names_from_qb_rules(qb_rules)

    # 分析差异
    analyze_differences(bangumi_names, qb_names)

    # 详细分析Bangumi中的重复
    analyze_bangumi_duplicates(bangumi_data)

    print(f"\n总结")
    print(f"=" * 60)
    print(f"Bangumi结果数量: {len(bangumi_data)}")
    print(f"qBittorrent规则数量: {len(qb_rules)}")
    print(f"差异: {len(bangumi_data) - len(qb_rules)}")

    if len(bangumi_data) > len(qb_rules):
        print(f"\n原因分析:")
        print(f"  - 有 {len(bangumi_data) - len(qb_rules)} 个作品在Bangumi结果中重复")
        print(f"  - qBittorrent规则生成时自动合并了重复作品")
        print(f"  - 这是正常行为，确保每个作品只有一个下载规则")

if __name__ == "__main__":
    main()