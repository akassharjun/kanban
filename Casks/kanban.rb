cask "kanban" do
  version "0.6.1"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "8962b009e9147a926a3a6b67ef2542af9161e5fe80a9b69348d5d7d699b6bae2"
  else
    sha256 "77f7515bf6e03ea6c9b7e837628ea9cc41c9e197e5de4d396609f6ca8890dde9"
  end

  name "Kanban"
  desc "Desktop project management for AI agent orchestration"
  homepage "https://github.com/akassharjun/kanban"

  app "Kanban.app"

  zap trash: [
    "~/.kanban",
    "~/Library/Caches/com.kanban.desktop",
    "~/Library/Preferences/com.kanban.desktop.plist",
    "~/Library/WebKit/com.kanban.desktop",
  ]
end
