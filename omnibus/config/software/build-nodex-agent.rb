name "build-nodex-agent"
default_version "1.0.0"

build do
  nodex_dir = File.expand_path('..', Omnibus::Config.project_root)
  unless Dir.exist?("#{project_dir}/agent")
    mkdir "#{project_dir}/agent"
  end
  copy "#{nodex_dir}/agent/*", "#{project_dir}/agent/"

  unless Dir.exist?("#{project_dir}/bin")
    mkdir "#{project_dir}/bin"
  end
  copy "#{nodex_dir}/bin/*", "#{project_dir}/bin/"

  unless Dir.exist?("#{project_dir}/controller")
    mkdir "#{project_dir}/controller"
  end
  copy "#{nodex_dir}/controller/*", "#{project_dir}/controller/"

  unless Dir.exist?("#{project_dir}/protocol")
    mkdir "#{project_dir}/protocol"
  end
  copy "#{nodex_dir}/protocol/*", "#{project_dir}/protocol/"

  unless Dir.exist?("#{project_dir}/e2e")
    mkdir "#{project_dir}/e2e"
  end
  copy "#{nodex_dir}/e2e/*", "#{project_dir}/e2e/"

  copy "#{nodex_dir}/Cargo.toml", "#{project_dir}"
  copy "#{nodex_dir}/Cargo.lock", "#{project_dir}"


  command "cd #{project_dir} && cross build --target #{ENV['TARGET_ARCH']} --release"
  copy "#{project_dir}/target/#{ENV['TARGET_ARCH']}/release/nodex-agent", "#{install_dir}/bin"
  if ENV['TARGET_PLATFORM'] == 'ubuntu'
    copy "#{nodex_dir}/omnibus/docs/deb/README.md", "#{install_dir}/README.md"
  end

  unless Dir.exist?("#{install_dir}/var/run")
    # prepare pid directory
    mkdir "#{install_dir}/var/run"
  end
end
