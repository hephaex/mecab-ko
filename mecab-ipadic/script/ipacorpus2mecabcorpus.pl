#!/usr/bin/perl

# �Ԥ�    ����    �Ԥ�    ư��-��Ω       ���ʡ�����¥����        ���ܷ�

# �Ĥ��� �Ĥ��� �Ĥ� ư�� * �Ҳ�ư�쥫�� ����Ϣ�ѥƷ�

while (<>) {
    chomp;
    next if (/^\*/ || /^#/);
    
    if (/EOS/) {
	print "EOS\n";
    } else {
	my @w = split;
	
	for my $i (0..6) {
	    $w[$i] = "*" if ($w[$i] eq "");
	}
	my @w2 = split /-/, $w[4];
	for my $i (0..3) {
	    $w2[$i] = "*" if (! defined $w2[$i])
	}

	print "$w[0]\t$w2[0],$w2[1],$w2[2],$w2[3],$w[5],$w[6],$w[3],$w[1],$w[2]\n";
    }
}  


