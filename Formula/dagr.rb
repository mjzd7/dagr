class Dagr < Formula
  desc "The DAG-Native Symbolic AST Slicing Hypervisor & Safety Sandbox for AI Coding Agents"
  homepage "https://github.com/mjzd7/dagr"
  url "https://github.com/mjzd7/dagr/archive/refs/tags/v0.1.0.tar.gz"
  head "https://github.com/mjzd7/dagr.git", branch: "main"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/dagr-cli")
  end

  test do
    assert_match "DAGR", shell_output("#{bin}/dagr --help")
  end
end
