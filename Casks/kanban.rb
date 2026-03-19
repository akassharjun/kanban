cask "kanban" do
  version "0.8.4"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "86f917202f8cdc85e4ad4b48431af147d4851e42fe4ced11c6ce376838530d7d"
  else
    sha256 "dba19068ae47bb21a2b6bcb70bd68404221d9ea51edfc6f86e420cac69faa4b0"
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
