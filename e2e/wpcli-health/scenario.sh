#!/bin/sh
# Runs inside the container. DB=1 starts MariaDB seeded like a real site (prefix wpx_,
# 2 MB of autoloaded options, wp-mail-smtp active) so the raw-DB fallback has data.
if [ "${DB:-0}" = 1 ]; then
  mysqld_safe >/dev/null 2>&1 &
  until mysqladmin ping >/dev/null 2>&1; do sleep 0.2; done
  mysql <<'SQL'
CREATE DATABASE wp;
CREATE USER 'wp'@'localhost' IDENTIFIED BY 'S3cr3t!pw';
GRANT SELECT ON wp.* TO 'wp'@'localhost';
CREATE TABLE wp.wpx_options (option_name VARCHAR(191), option_value LONGTEXT, autoload VARCHAR(20));
INSERT INTO wp.wpx_options VALUES
  ('active_plugins', 'a:2:{i:0;s:29:"wp-mail-smtp/wp_mail_smtp.php";i:1;s:19:"akismet/akismet.php";}', 'yes'),
  ('big_blob', REPEAT('x', 2097152), 'yes');
SQL
fi
cd /var/www/html && exec wp-agent "$@"
