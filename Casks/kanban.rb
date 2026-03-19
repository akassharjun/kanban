cask "kanban" do
  version "0.9.0"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "a9e533d8e055be1caa4bfe8f609ee6d68646c12d55c63f0d570a8f44795d80d4"
  else
    sha256 "e89a9a6fb42f4c138e9afdd6f33914b9321de762917a645e6c73742b7bd2d788"
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
