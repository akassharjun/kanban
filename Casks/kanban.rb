cask "kanban" do
  version "0.4.0"
  arch arm: "aarch64", intel: "x64"

  url "https://github.com/akassharjun/kanban/releases/download/v#{version}/Kanban_#{version}_#{arch}.dmg"
  if Hardware::CPU.arm?
    sha256 "21d9bf64383931054f4ecc680c5f22302b114bb051557e3d81d810c5fffd8b07"
  else
    sha256 "b37396f234d1e9c97604a6681f07f6e612dd4acf52b9deebd5fe02aeb24944bb"
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
