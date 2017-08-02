package Sexp;
use Carp;
use IO::File;
use strict;
use base qw/ Exporter /;
use vars qw/ @EXPORT_OK /;
@EXPORT_OK = qw/ parse /;

=head1 NAME

Juman::Sexp - S�����ɤ߹���⥸�塼��

=head1 SYNOPSIS

 use Data::Dumper;
 use Juman::Sexp qw/ parse /;
 print &Dumper( &parse( file => "Noun.dic" ) );

=head1 DESCRIPTION

C<Juman::Sexp> �ϡ�Juman ���������ե�������Ѥ����Ƥ���S�����ɤ߹�
�ि��δؿ� C<parse> ��������Ƥ��롥

=head1 FUNCTIONS

=over 4

=item parse

���ꤵ�줿�оݤ�S���Ȥ��Ʋ��Ϥ���ؿ����ʲ��Υ��ץ���������դ��롥

=over 4

=item file => FILE

���Ϥ���ե��������ꤹ�롥

=item string => STRING

���Ϥ���ʸ�������ꤹ�롥

=item comment => STRING

�����ȳ���ʸ�������ꤹ�롥�����Ȥ�ޤä����ޤޤʤ��оݤ���Ϥ���
���ϡ��ʲ��Τ褦��̤����ͤ���ꤹ�롥

  Example:

    &parse( file => "example.dat", comment => undef );

=item debug => BOOLEAN

�ǥХå��Ѥξ������Ϥ���褦�˻ؼ����롥

=back

=back

�㤨�С�ʸ������оݤȤ��Ʋ��Ϥ�����ϡ��ʲ��Τ褦�˻��ꤹ�롥

  Example:

    &parse( string =>
            "(̾�� (����̾�� ((�ɤ� ����)(���Ф��� �� ���� ����))))" );

���ξ�硤���Τ褦�ʲ��Ϸ�̤��֤���롥

    ( [ '̾��',
         [ '����̾��',
           [ [ '�ɤ�', '����' ],
             [ '���Ф���', '��', '����', '����' ]
           ]
         ]
       ] )

=cut
sub parse {
    my %option;
    $option{comment} = ";";
    while( @_ ){
	my $key = shift;
	my $val = shift;
	$key =~ s/\A-+//;
	$option{lc($key)} = $val;
    }
    if( $option{file} ){
	my $file = $option{file};
	if( my $fh = IO::File->new( $file, "r" ) ){
	    my $num = 0;
	    &_parse( sub { if( $fh->eof ){ undef; } else { $num++; $fh->getline; } },
		     sub { "at $file line $num"; },
		     $option{comment},
		     $option{debug} );
	} else {
	    warn "Cannot open $file: $!\n";
	    wantarray ? () : 0;
	}
    } elsif( $option{string} ){
	my $string = $option{string};
	&_parse( sub { my $x = $string; $string = undef; $x; },
		 sub { "in string"; },
		 $option{comment},
		 $option{debug} );
    } else {
	carp "Neither `file' option nor `string' option is specified";
	wantarray ? () : 0;
    }
}

sub _parse {
    my( $getline, $place, $comment, $debug ) = @_;
    my $str = "";
    my @stack;		# shift-reduce ˡ�ǹ�ʸ���Ϥ��뤿��Υ����å� 
    my @offset;		# reduce ���٤����ǿ���Ͽ���Ƥ���������
    while(1){
	$str =~ s/\A\s*//;
	$str =~ s/\A$comment[^\n]*\n\s*// if $comment;
	if( ! $str ){
	    if( $str = &$getline() ){
		print STDERR "PARSE: $str" if $debug;
	    } else {
		if( @offset ){
		    die "Syntax error: end of target during parsing.\n";
		} else {
		    last;
		}
	    }
	}
	# ����̤� shift ����
	elsif( $str =~ s/\A\(// ){
	    $offset[0]-- if @offset;
	    unshift( @offset, 0 );
	}
	# ʸ����� shift ����
	elsif( $str =~ m/\A"/ ){
	    while(1){
		if( $str =~ s/\A("(?:[^"\\]+|\\.)*")// ){
		    $offset[0]--;
		    push( @stack, $1 );
		    last;
		} elsif( my $next = &$getline() ){
		    $str .= $next;
		} else {
		    die "Syntax error: end of target during string.\n";
		}
	    }
	}
	# ����ܥ�� shift ����
	elsif( $str =~ s/\A([^\s"()]+)// ){
	    $offset[0]--;
	    push( @stack, $1 );
	}
	# �ĳ��(= �ꥹ��)�� reduce ����
	elsif( $str =~ s/\A\)// ){
	    unless( @offset ){
		die( "Syntax error: too much close brackets ", &$place(), ".\n" );
	    } else {
		my $offset = shift @offset;
		if( $offset < 0 ){
		    push( @stack, [ splice( @stack, $offset ) ] );
		} else {
		    push( @stack, [] );
		}
	    }
	}
	else {
	    die( "Syntax error: unknown syntax element ", &$place(), ".\n" );
	}
    }
    @stack;
}

1;

=head1 AUTHOR

=over 4

=item
TSUCHIYA Masatoshi <tsuchiya@pine.kuee.kyoto-u.ac.jp>

=back

=head1 COPYRIGHT

���ѵڤӺ����ۤˤĤ��Ƥ� GPL2 �ޤ��� Artistic License �˽��äƤ���������

=cut

__END__
# Local Variables:
# mode: perl
# coding: euc-japan
# use-kuten-for-period: nil
# use-touten-for-comma: nil
# End:
