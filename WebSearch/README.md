# WebSearch

Take a Panel 网页搜索插件。输入 `web` 并按 Enter 打开搜索栏，输入关键词后按 Enter 或点击「搜索」，使用系统默认浏览器打开结果。也可输入 `web 关键词` 预填搜索内容。输入停顿 300 毫秒后，在「搜索推荐」区域显示所选引擎的建议，点击推荐词即可搜索。无推荐或请求失败时仍可按 Enter 搜索。Esc 或「返回」退出搜索栏。

在「设置 → 插件」中选择 WebSearch，选择搜索引擎并保存。支持 Bing（默认）、Google、百度、DuckDuckGo。浏览器由操作系统的默认浏览器设置决定。

## 安装

将插件压缩包解压到长期保留的位置，在 Take a Panel 的「设置 → 插件」点击「浏览…」，选择其中的 `manifest.json`，添加并启用。请保留清单与 `bin` 目录。

## 构建和打包

安装 Rust 1.92 或更新版本及 Python 3，在本目录执行：

```sh
python3 package.py
```

脚本构建当前系统和架构的 release 程序，放入 `bin`，并在 `dist` 生成可安装的 ZIP。Windows 可用 `python package.py`，ZIP 清单自动指向 `.exe`。安装者无需 Rust 或 Python；推荐功能需要 `curl` 和网络，Linux 还需要 `xdg-open`。需要使用支持 `suggestions` 字段和 `panel_change` 的新版主程序。

验证：`cargo test --locked`。测试不会启动浏览器。输入内容会发送给所选搜索引擎的建议接口；搜索内容会通过 URL 交给所选搜索引擎，最多 4096 UTF-8 字节。
