class Shiki < Formula
  desc "TUI note-taking app with a Yazi-inspired three-pane layout and git-backed notebooks"
  homepage "https://github.com/sazardev/shiki"
  version "0.9.7"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/sazardev/shiki/releases/download/v0.9.7/shiki-v0.9.7-aarch64-apple-darwin.tar.gz"
      sha256 "46caeef0ecc88b73041f07882d6e6d4d949af6bda20e5a2b68e9c559f10bacd5"
    end
    on_intel do
      url "https://github.com/sazardev/shiki/releases/download/v0.9.7/shiki-v0.9.7-x86_64-apple-darwin.tar.gz"
      sha256 "438f4786b01ee1f1c6c73747aaa120231516ec2bdb6f311eae5859fca7222a2a"
    end
  end

  def install
    bin.install "shiki"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/shiki --version")
  end
end
