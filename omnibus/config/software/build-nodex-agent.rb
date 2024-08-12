name "build-nodex-agent"
default_version "1.0.0"

# dependency "zlib"

build do
  nodex_dir = File.expand_path('..', Omnibus::Config.project_root)
  unless Dir.exist?("#{project_dir}/src")
    mkdir "#{project_dir}/src"
  end
  copy "#{nodex_dir}/src/*", "#{project_dir}/src/"

  unless Dir.exist?("#{project_dir}/e2e")
    mkdir "#{project_dir}/e2e"
  end
  copy "#{nodex_dir}/e2e/*", "#{project_dir}/e2e/"

  copy "#{nodex_dir}/build.rs", "#{project_dir}"
  copy "#{nodex_dir}/Cargo.toml", "#{project_dir}"
  copy "#{nodex_dir}/Cargo.lock", "#{project_dir}"

  # env = with_standard_compiler_flags(with_embedded_path)
  # command "cd #{project_dir} && cargo build --release", env: env
  command "cd #{project_dir} && cargo build --release"

  # unless Dir.exist?("#{install_dir}/bin")
  #   mkdir "#{install_dir}/bin"
  # end
  copy "#{project_dir}/target/release/nodex-agent", "#{install_dir}/bin"
  # command "ldd #{install_dir}/bin/nodex-agent | grep '=>' | awk '{print $3}' | xargs -I {} cp {} #{install_dir}/embedded/lib/"

  unless Dir.exist?("#{install_dir}/var/run")
    # prepare pid directory
    mkdir "#{install_dir}/var/run"
  end
end
