#!/bin/bash

# 构建语言服务器的所有平台版本
# 注意：交叉编译需要安装相应的工具链，推荐使用 CI/CD 在各平台上分别构建

set -e

echo "🔨 Building Summer LSP server for all platforms..."
echo ""
echo "⚠️  注意：交叉编译需要安装工具链，可能会失败。"
echo "   推荐方案："
echo "   1. 开发时使用: npm run build:server:current"
echo "   2. 发布时使用 CI/CD 在各平台上分别构建"
echo ""
read -p "继续构建所有平台？(y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "已取消。使用 'npm run build:server:current' 构建当前平台。"
  exit 0
fi

# 进入项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../.."

# 定义目标平台
TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-msvc"
)

# 创建 bin 目录
mkdir -p vscode/bin

# 检测当前平台
CURRENT_OS=$(uname -s)
CURRENT_ARCH=$(uname -m)

echo "当前平台: $CURRENT_OS $CURRENT_ARCH"
echo ""

# 构建每个平台
for target in "${TARGETS[@]}"; do
  echo ""
  echo "📦 Building for $target..."
  
  # 检查是否是当前平台（可以直接构建）
  CAN_BUILD=false
  case "$target" in
    *linux*)
      [ "$CURRENT_OS" = "Linux" ] && CAN_BUILD=true
      ;;
    *darwin*)
      if [ "$CURRENT_OS" = "Darwin" ]; then
        if [[ "$target" == *"aarch64"* ]]; then
          [ "$CURRENT_ARCH" = "arm64" ] && CAN_BUILD=true
        else
          [ "$CURRENT_ARCH" = "x86_64" ] && CAN_BUILD=true
        fi
      fi
      ;;
    *windows*)
      [[ "$CURRENT_OS" == MINGW* ]] || [[ "$CURRENT_OS" == MSYS* ]] && CAN_BUILD=true
      ;;
  esac
  
  if [ "$CAN_BUILD" = false ]; then
    echo "  ⚠️  跳过 $target (需要交叉编译工具链)"
    echo "     在 CI/CD 中使用对应平台构建此目标"
    continue
  fi
  
  # 检查是否已安装目标
  if ! rustup target list | grep -q "$target (installed)"; then
    echo "  Installing target $target..."
    rustup target add "$target"
  fi
  
  # 构建
  echo "  Compiling..."
  if cargo build --release --target "$target"; then
    # 复制到 bin 目录
    case "$target" in
      *linux*)
        cp "target/$target/release/summer-lsp" "vscode/bin/summer-lsp-linux-x64"
        echo "  ✅ Copied to vscode/bin/summer-lsp-linux-x64"
        ;;
      *darwin*)
        if [[ "$target" == *"aarch64"* ]]; then
          cp "target/$target/release/summer-lsp" "vscode/bin/summer-lsp-darwin-arm64"
          echo "  ✅ Copied to vscode/bin/summer-lsp-darwin-arm64"
        else
          cp "target/$target/release/summer-lsp" "vscode/bin/summer-lsp-darwin-x64"
          echo "  ✅ Copied to vscode/bin/summer-lsp-darwin-x64"
        fi
        ;;
      *windows*)
        cp "target/$target/release/summer-lsp.exe" "vscode/bin/summer-lsp-win32-x64.exe"
        echo "  ✅ Copied to vscode/bin/summer-lsp-win32-x64.exe"
        ;;
    esac
  else
    echo "  ❌ Failed to build $target"
    echo "     这是正常的，交叉编译需要额外的工具链"
  fi
done

echo ""
echo "✅ 构建完成！"
echo ""
echo "📦 已构建的二进制文件:"
ls -lh vscode/bin/ 2>/dev/null || echo "  (无)"
echo ""
echo "💡 提示："
echo "   - 开发时使用: npm run build:server:current"
echo "   - 发布时使用 CI/CD 在各平台上分别构建"

