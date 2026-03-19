cask "kanban" do
  version "0.8.8"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "fa3e13a11afecf747a63062e677652f68fb43c7f8d9875bad318bd0c8e1a3d8f"
  else
    sha256 "fc8d28e109c1786ed1b27a58cbf6d64999d868b1e35967b58b6326f05b8c3b04"
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
