class Shiki < Formula
  desc "TUI note-taking app with a Yazi-inspired three-pane layout and git-backed notebooks"
  homepage "https://github.com/sazardev/shiki"
  version "0.10.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/sazardev/shiki/releases/download/v0.10.0/shiki-v0.10.0-aarch64-apple-darwin.tar.gz"
      sha256 "0441b56ab20895aa053fc27b521c4a275797fcf149ac344da3862e0e0367bfff"
    end
    on_intel do
      url "https://github.com/sazardev/shiki/releases/download/v0.10.0/shiki-v0.10.0-x86_64-apple-darwin.tar.gz"
      sha256 "4f5da2e8e026bf5645b4f3ec729aa613f7e9fe36b44635e033bb943346dc7e36"
    end
  end

  def install
    bin.install "shiki"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/shiki --version")
  end
end
