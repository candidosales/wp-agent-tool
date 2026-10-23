#!/bin/sh
# Runs inside the container. Usage: scenario.sh exposed|direct|stuck|clean [wp-agent args]
s=$1; shift
recent=$(date -u -d '10 days ago' '+%Y-%m-%d %H:%M:%S')
old='{"user_login":"owner","user_email":"owner@example.com","user_registered":"2019-03-01 10:00:00","roles":"administrator"}'
export PLUGINS="akismet" USERS="[$old]"
up=/var/www/html/wp-content/uploads
case "$s" in
  exposed)
    export USERS="[$old,{\"user_login\":\"intruder\",\"user_email\":\"x@evil.test\",\"user_registered\":\"$recent\",\"roles\":\"administrator\"}]"
    echo 'CREATE TABLE wp_users;' > $up/2024/01/site-backup.sql
    echo zip > $up/2024/01/fotos.zip
    mkdir -p $up/wpallexport/exports/907d && echo a,b > $up/wpallexport/exports/907d/Posts-Export.csv
    echo 'Deny from all' > $up/wpallexport/exports/907d/.htaccess
    # No MTA at all.
    rm -f /usr/sbin/sendmail /usr/lib/sendmail /usr/sbin/postconf
    ;;
  direct)
    postfix start 2>/dev/null
    ;;
  stuck)
    # Geleia's setup: relay via Gmail with SASL, but the credentials file was never created.
    postconf -e 'relayhost=[127.0.0.1]:2525' smtp_sasl_auth_enable=yes smtp_sasl_password_maps=hash:/etc/postfix/sasl_passwd
    postfix start 2>/dev/null
    echo "Subject: reset" | sendmail someone@gmail.com
    for _ in $(seq 30); do postqueue -j | grep -q delay_reason && break; sleep 1; done
    ;;
  clean)
    export PLUGINS="akismet,wp-mail-smtp"
    ;;
esac
cd /var/www/html && wp-agent "$@"
echo "exit=$?" >&2
