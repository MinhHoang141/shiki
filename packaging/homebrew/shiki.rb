class Shiki < Formula
  desc "TUI note-taking app with a Yazi-inspired three-pane layout and git-backed notebooks"
  homepage "https://github.com/sazardev/shiki"
  version "0.9.6"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/sazardev/shiki/releases/download/v0.9.6/shiki-v0.9.6-aarch64-apple-darwin.tar.gz"
      sha256 "4eeff2b07cb7ef8aa31e665a47265ef6849f665ab2e7c85bb0ae170a9667c43d"
    end
    on_intel do
      url "https://github.com/sazardev/shiki/releases/download/v0.9.6/shiki-v0.9.6-x86_64-apple-darwin.tar.gz"
      sha256 "91e7f151e855db35f52e0edaccbee4ea5d469cc31f0022dccd8a7450a1dc68a9"
    end
  end

  def install
    bin.install "shiki"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/shiki --version")
  end
end
