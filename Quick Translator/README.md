# Quick Translator

Take a Panel 的翻译插件。在主面板输入 `tr` 并按 Enter 打开翻译界面；也可输入 `tr 要翻译的文字` 预填原文。输入后按 Enter 翻译，Shift+Enter 换行。源语言和目标语言可在面板下拉菜单中选择，也可在插件设置中保存默认方向。

## 安装已构建的插件

1. 下载与操作系统和处理器架构相匹配的插件压缩包，并解压到希望长期保存的位置。
2. 在 Take a Panel 的「设置 → 插件」中输入解压目录内 `manifest.json` 的完整路径，点击「添加插件」。
3. 输入 `tr`，等待「打开翻译面板」出现后按 Enter。

解压后的目录需要保留 `manifest.json` 和 `bin` 文件夹及其中的程序；不要移动或删除这些文件。安装者无需 Rust。插件要求 Take a Panel 支持插件面板，并需要 `curl` 和网络连接。

## 从源码构建和打包

在本目录安装 Rust 1.92 或更新版本及 Python 3，然后执行：

```sh
python3 package.py
```

Windows 可执行 `python package.py`。脚本在插件目录的 `target` 中构建当前系统和架构的 release 版本，将程序放入 `bin`，并在 `dist` 中生成带架构名称的 ZIP。Windows ZIP 内的清单会自动指向 `.exe`。请在目标平台运行脚本以生成对应平台的包。

翻译使用 [MyMemory API](https://mymemory.translated.net/doc/spec.php)，会将输入的文字发送给该服务。单次原文最多 500 UTF-8 字节，服务有使用额度限制。语言是否支持取决于服务。
