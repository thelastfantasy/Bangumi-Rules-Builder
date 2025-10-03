#!/usr/bin/env python3
"""
qBittorrent Rule Editor
一个简单的UI工具，用于编辑生成的qBittorrent RSS下载规则文件
"""

import json
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
from pathlib import Path
from typing import Dict, Any, List


class QBRuleEditor:
    def __init__(self, root):
        self.root = root
        self.root.title("qBittorrent 规则编辑器")
        self.root.geometry("800x600")

        # 窗口居中显示
        self.center_window()

        # 规则文件路径
        self.original_rules_file = Path("qb_download_rules.json")
        self.modified_rules_file = Path("qb_download_rules_mod.json")

        # 当前使用的文件路径
        self.current_rules_file = self.original_rules_file

        # 存储当前规则数据
        self.rules_data: Dict[str, Any] = {}

        # 存储要删除的规则
        self.rules_to_delete: List[str] = []

        self.setup_ui()
        self.load_rules()

    def _on_mousewheel(self, event):
        """鼠标滚轮事件处理"""
        # Windows和macOS
        if event.delta:
            self.canvas.yview_scroll(int(-1 * (event.delta / 120)), "units")
        # Linux
        elif event.num == 4:
            self.canvas.yview_scroll(-1, "units")
        elif event.num == 5:
            self.canvas.yview_scroll(1, "units")

    def _bind_to_children(self, widget):
        """递归绑定滚轮事件到所有子组件"""
        for child in widget.winfo_children():
            child.bind("<MouseWheel>", self._on_mousewheel)
            child.bind("<Button-4>", self._on_mousewheel)
            child.bind("<Button-5>", self._on_mousewheel)
            self._bind_to_children(child)

    def center_window(self):
        """将窗口居中显示在屏幕上"""
        # 更新窗口以确保获取正确的尺寸
        self.root.update_idletasks()

        # 获取窗口尺寸
        width = self.root.winfo_width()
        height = self.root.winfo_height()

        # 获取屏幕尺寸
        screen_width = self.root.winfo_screenwidth()
        screen_height = self.root.winfo_screenheight()

        # 计算居中位置
        x = (screen_width - width) // 2
        y = (screen_height - height) // 2

        # 设置窗口位置
        self.root.geometry(f"{width}x{height}+{x}+{y}")


    def setup_ui(self):
        """设置用户界面"""
        # 主框架
        main_frame = ttk.Frame(self.root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))

        # 配置网格权重
        self.root.columnconfigure(0, weight=1)
        self.root.rowconfigure(0, weight=1)
        main_frame.columnconfigure(1, weight=1)
        main_frame.rowconfigure(1, weight=1)

        # 顶部按钮框架
        button_frame = ttk.Frame(main_frame)
        button_frame.grid(row=0, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(0, 10))

        # 刷新按钮
        refresh_btn = ttk.Button(
            button_frame,
            text="🔄 刷新",
            command=self.load_rules
        )
        refresh_btn.pack(side=tk.LEFT, padx=(0, 10))

        # 选择文件按钮
        file_btn = ttk.Button(
            button_frame,
            text="📁 选择文件",
            command=self.select_file
        )
        file_btn.pack(side=tk.LEFT, padx=(0, 10))

        # 复制按钮
        copy_btn = ttk.Button(
            button_frame,
            text="📋 复制",
            command=self.copy_rules_to_clipboard
        )
        copy_btn.pack(side=tk.LEFT, padx=(0, 10))

        # 保存按钮
        save_btn = ttk.Button(
            button_frame,
            text="💾 保存",
            command=self.save_rules,
            style="Accent.TButton"
        )
        save_btn.pack(side=tk.LEFT)

        # 状态标签
        self.status_label = ttk.Label(button_frame, text="")
        self.status_label.pack(side=tk.RIGHT)

        # 文件信息标签
        self.file_info_label = ttk.Label(button_frame, text="", foreground="blue")
        self.file_info_label.pack(side=tk.RIGHT, padx=(0, 10))

        # 规则列表框架
        list_frame = ttk.LabelFrame(main_frame, text="番组列表 (点击删除按钮移除不感兴趣的番组)")
        list_frame.grid(row=1, column=0, columnspan=2, sticky=(tk.W, tk.E, tk.N, tk.S), pady=(0, 10))
        list_frame.columnconfigure(0, weight=1)
        list_frame.rowconfigure(0, weight=1)

        # 创建滚动框架 - 使用双缓冲优化
        self.canvas = tk.Canvas(list_frame, bg='white')

        # 双缓冲优化 - 减少滚动时的闪烁
        self.canvas.config(highlightthickness=0)

        # 优化滚动性能 - 使用更平滑的滚动
        def smooth_scroll(*args):
            self.canvas.yview(*args)
            # 强制Canvas立即重绘
            self.canvas.update_idletasks()

        scrollbar = ttk.Scrollbar(list_frame, orient="vertical", command=smooth_scroll)
        self.scrollable_frame = ttk.Frame(self.canvas)

        self.scrollable_frame.bind(
            "<Configure>",
            lambda e: self.canvas.configure(scrollregion=self.canvas.bbox("all"))
        )

        self.canvas.create_window((0, 0), window=self.scrollable_frame, anchor="nw")
        self.canvas.configure(yscrollcommand=scrollbar.set)

        # 绑定鼠标滚轮事件
        self.canvas.bind("<MouseWheel>", self._on_mousewheel)
        self.canvas.bind("<Button-4>", self._on_mousewheel)
        self.canvas.bind("<Button-5>", self._on_mousewheel)

        # 绑定到滚动框架
        self.scrollable_frame.bind("<MouseWheel>", self._on_mousewheel)
        self.scrollable_frame.bind("<Button-4>", self._on_mousewheel)
        self.scrollable_frame.bind("<Button-5>", self._on_mousewheel)

        # 初始绑定所有子组件
        self._bind_to_children(self.scrollable_frame)

        # 当内容更新时重新绑定
        self.scrollable_frame.bind(
            "<Configure>",
            lambda e: (self.canvas.configure(scrollregion=self.canvas.bbox("all")),
                      self._bind_to_children(self.scrollable_frame))
        )

        self.canvas.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        scrollbar.grid(row=0, column=1, sticky=(tk.N, tk.S))

        list_frame.columnconfigure(0, weight=1)
        list_frame.rowconfigure(0, weight=1)

        # 底部信息框架
        info_frame = ttk.Frame(main_frame)
        info_frame.grid(row=2, column=0, columnspan=2, sticky=(tk.W, tk.E))

        # 规则数量显示
        self.count_label = ttk.Label(info_frame, text="")
        self.count_label.pack(side=tk.LEFT)

        # 删除数量显示
        self.delete_label = ttk.Label(info_frame, text="", foreground="red")
        self.delete_label.pack(side=tk.RIGHT)

    def select_file(self):
        """选择规则文件"""
        file_path = filedialog.askopenfilename(
            title="选择qBittorrent规则文件",
            filetypes=[("JSON文件", "*.json"), ("所有文件", "*.*")],
            initialdir="."
        )
        if file_path:
            self.original_rules_file = Path(file_path)
            self.load_rules()

    def load_rules(self):
        """加载规则文件 - 优先加载mod文件，不存在则加载原文件"""
        try:
            # 优先尝试加载mod文件
            if self.modified_rules_file.exists():
                self.current_rules_file = self.modified_rules_file
                with open(self.modified_rules_file, 'r', encoding='utf-8') as f:
                    self.rules_data = json.load(f)
                self.update_status("✅ 已加载修改后的规则文件")
                self.file_info_label.config(text="📁 修改文件")
            elif self.original_rules_file.exists():
                self.current_rules_file = self.original_rules_file
                with open(self.original_rules_file, 'r', encoding='utf-8') as f:
                    self.rules_data = json.load(f)
                self.update_status("✅ 已加载原始规则文件")
                self.file_info_label.config(text="📁 原始文件")
            else:
                messagebox.showerror("错误", f"规则文件不存在: {self.original_rules_file}")
                return

            self.rules_to_delete.clear()
            self.update_rule_list()

        except Exception as e:
            messagebox.showerror("错误", f"加载规则文件失败: {str(e)}")

    def update_rule_list(self):
        """更新规则列表显示"""
        # 清空现有内容
        for widget in self.scrollable_frame.winfo_children():
            widget.destroy()

        if not self.rules_data:
            empty_label = ttk.Label(self.scrollable_frame, text="没有找到规则数据")
            empty_label.pack(pady=20)
            # 更新滚动区域
            self.canvas.configure(scrollregion=self.canvas.bbox("all"))
            return

        # 按规则名称排序
        sorted_rules = sorted(self.rules_data.keys())

        for i, rule_name in enumerate(sorted_rules):
            rule_frame = ttk.Frame(self.scrollable_frame)
            rule_frame.pack(fill=tk.X, padx=5, pady=2)

            # 检查是否标记为删除
            is_marked_for_deletion = rule_name in self.rules_to_delete

            # 删除按钮 - 根据状态显示不同文本
            delete_btn_text = "✅" if is_marked_for_deletion else "❌"
            delete_btn = ttk.Button(
                rule_frame,
                text=delete_btn_text,
                width=4,
                command=lambda name=rule_name: self.mark_for_deletion(name)
            )
            delete_btn.pack(side=tk.LEFT, padx=(0, 10))

            # 规则名称标签 - 根据状态设置不同样式
            rule_label = ttk.Label(rule_frame, text=rule_name, anchor="w")
            if is_marked_for_deletion:
                # 标记删除的规则显示为灰色带删除线效果
                rule_label.config(foreground="gray")
            rule_label.pack(side=tk.LEFT, fill=tk.X, expand=True)

            # 添加分隔线（除了最后一个）
            if i < len(sorted_rules) - 1:
                separator = ttk.Separator(self.scrollable_frame, orient="horizontal")
                separator.pack(fill=tk.X, padx=5, pady=2)

        # 更新滚动区域
        self.canvas.configure(scrollregion=self.canvas.bbox("all"))

        # 重新绑定滚轮事件到新创建的组件
        self._bind_to_children(self.scrollable_frame)

        # 更新计数
        total_rules = len(self.rules_data)
        delete_count = len(self.rules_to_delete)
        self.count_label.config(text=f"总规则数: {total_rules}")
        if delete_count > 0:
            self.delete_label.config(text=f"待删除: {delete_count}")
        else:
            self.delete_label.config(text="")

    def mark_for_deletion(self, rule_name: str):
        """标记规则为待删除"""
        # 保存当前滚动位置
        scroll_position = self.canvas.yview()

        if rule_name in self.rules_to_delete:
            # 如果已经标记，则取消标记
            self.rules_to_delete.remove(rule_name)
            self.update_status(f"✅ 已取消删除: {rule_name}")
        else:
            # 标记为待删除
            self.rules_to_delete.append(rule_name)
            self.update_status(f"🗑️ 已标记删除: {rule_name}")

        # 更新UI以反映新的删除状态 - 使用更优化的方式
        self._update_single_rule_display(rule_name)

        # 更新删除计数
        delete_count = len(self.rules_to_delete)
        if delete_count > 0:
            self.delete_label.config(text=f"待删除: {delete_count}")
        else:
            self.delete_label.config(text="")

    def _update_single_rule_display(self, rule_name: str):
        """只更新单个规则的显示状态，避免整个列表重绘"""
        # 遍历所有规则框架，找到对应的规则并更新
        for widget in self.scrollable_frame.winfo_children():
            if isinstance(widget, ttk.Frame):
                # 查找规则名称标签
                for child in widget.winfo_children():
                    if isinstance(child, ttk.Label):
                        # 检查标签文本是否匹配规则名称
                        if child.cget("text") == rule_name:
                            # 找到对应的规则框架
                            is_marked_for_deletion = rule_name in self.rules_to_delete

                            # 更新删除按钮
                            for btn in widget.winfo_children():
                                if isinstance(btn, ttk.Button):
                                    btn_text = "✅" if is_marked_for_deletion else "❌"
                                    btn.config(text=btn_text)
                                    break

                            # 更新标签颜色
                            if is_marked_for_deletion:
                                child.config(foreground="gray")
                            else:
                                child.config(foreground="")

                            # 找到匹配项后退出
                            return

    def save_rules(self):
        """保存修改后的规则文件到mod文件"""
        if not self.rules_to_delete:
            messagebox.showinfo("信息", "没有要删除的规则")
            return

        # 确认删除
        confirm = messagebox.askyesno(
            "确认删除",
            f"确定要删除以下 {len(self.rules_to_delete)} 个规则吗？\n\n" +
            "\n".join(f"• {name}" for name in self.rules_to_delete)
        )

        if not confirm:
            return

        try:
            # 删除标记的规则
            new_rules_data = {}
            deleted_count = 0

            for rule_name, rule_data in self.rules_data.items():
                if rule_name not in self.rules_to_delete:
                    new_rules_data[rule_name] = rule_data
                else:
                    deleted_count += 1

            # 保存到mod文件
            with open(self.modified_rules_file, 'w', encoding='utf-8') as f:
                json.dump(new_rules_data, f, ensure_ascii=False, indent=2)

            # 更新数据
            self.rules_data = new_rules_data
            self.current_rules_file = self.modified_rules_file
            self.rules_to_delete.clear()

            self.update_rule_list()
            self.update_status(f"✅ 成功删除 {deleted_count} 个规则，已保存到修改文件")
            self.file_info_label.config(text="📁 修改文件")

            messagebox.showinfo(
                "保存成功",
                f"已成功删除 {deleted_count} 个规则。\n\n"
                f"修改已保存到: {self.modified_rules_file.name}\n"
                f"原始文件保持不变: {self.original_rules_file.name}"
            )

        except Exception as e:
            messagebox.showerror("错误", f"保存规则文件失败: {str(e)}")

    def copy_rules_to_clipboard(self):
        """复制当前UI显示的内容（排除已标记删除的规则）到剪贴板"""
        try:
            # 创建排除已标记删除规则的新数据
            filtered_rules = {}
            for rule_name, rule_data in self.rules_data.items():
                if rule_name not in self.rules_to_delete:
                    filtered_rules[rule_name] = rule_data

            # 转换为JSON字符串
            rules_json = json.dumps(filtered_rules, ensure_ascii=False, indent=2)

            # 复制到剪贴板
            self.root.clipboard_clear()
            self.root.clipboard_append(rules_json)

            # 显示成功信息
            remaining_count = len(filtered_rules)
            deleted_count = len(self.rules_to_delete)

            if deleted_count > 0:
                self.update_status(f"📋 已复制 {remaining_count} 个规则（已排除 {deleted_count} 个标记删除的规则）")
            else:
                self.update_status(f"📋 已复制 {remaining_count} 个规则到剪贴板")

            messagebox.showinfo(
                "复制成功",
                f"已成功复制 {remaining_count} 个规则到剪贴板。\n\n"
                f"当前标记删除的 {deleted_count} 个规则已被排除。\n\n"
                f"您可以将此JSON粘贴到任何文本编辑器中。"
            )

        except Exception as e:
            messagebox.showerror("错误", f"复制到剪贴板失败: {str(e)}")

    def update_status(self, message: str):
        """更新状态标签"""
        self.status_label.config(text=message)
        # 3秒后清除状态消息
        self.root.after(3000, lambda: self.status_label.config(text=""))


def main():
    """主函数"""
    root = tk.Tk()
    app = QBRuleEditor(root)
    root.mainloop()


if __name__ == "__main__":
    main()