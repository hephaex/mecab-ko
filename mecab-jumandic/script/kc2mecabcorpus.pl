#!/usr/bin/perl

# �� ���� * ��Ƭ�� ̾����Ƭ��
#  ������ ������ ������ ư�� * �Ҳ�ư���� ����Ϣ�ѷ�
while (<>) {
    chomp;
    next if (/^#/);
    next if (/^\*/);
    if (/EOS/) {
	print "EOS\n";
    } else {
	my @t = split /\s+/, $_;
	$t[2] = $t[0] if ($t[2] eq "*");
        print "$t[0]\t$t[3],$t[4],$t[5],$t[6],$t[2],$t[1]\n";
    }
}
