cask "kanban" do
  version "0.6.0"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "c17354af1b59cd35419258043ba1794d52f7b5f255937c19a8f5d20a579fbb1b"
  else
    sha256 "8061f8ba2ebb57cbd802646ae7ebce6c41967a92361ee801674be3ddeca6ee0e"
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
