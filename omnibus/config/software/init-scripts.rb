name "init-scripts"
# always_build true

ohai = Ohai::System.new
ohai.all_plugins

build do
  etc_dir = "/etc/nodex-agent"
  if ohai['platform_family'] == 'debian' || ohai['platform_family'] == 'rhel'
    mkdir "/etc/init"
    if ohai['platform_family'] == 'debian'
      # sysvinit support for debian only for now
      mkdir "/etc/init.d"
      # debian recommends using a different directory for systemd unit files
      systemd_directory = "/lib/systemd/system"
      erb source: "upstart_debian.conf.erb",
          dest: "/etc/init/nodex-agent.conf",
          mode: 0644,
          vars: { install_dir: install_dir, etc_dir: etc_dir }

      erb source: "sysvinit_debian.erb",
          dest: "/etc/init.d/datadog-agent",
          mode: 0755,
          vars: { install_dir: install_dir, etc_dir: etc_dir }

      project.extra_package_file '/etc/init.d/datadog-agent'

      erb source: "systemd.service.erb",
          dest: "#{systemd_directory}/nodex-agent.service",
          mode: 0644,
          vars: { install_dir: install_dir, etc_dir: etc_dir }
    end
  end
end
