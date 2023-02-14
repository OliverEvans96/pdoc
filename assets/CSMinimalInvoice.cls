%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Minimal Invoice
% LaTeX Class
% Version 1.1 (April 22, 2022)
%
% This class originates from:
% https://www.LaTeXTemplates.com
%
% Author:
% Vel (vel@latextemplates.com)
%
% License:
% CC BY-NC-SA 4.0 (https://creativecommons.org/licenses/by-nc-sa/4.0/)
%
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

%----------------------------------------------------------------------------------------
%	CLASS CONFIGURATION
%----------------------------------------------------------------------------------------

\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{CSMinimalInvoice}[2022/04/22 Creodocs Minimal Invoice Class v1.1]

\DeclareOption*{\PassOptionsToClass{\CurrentOption}{extarticle}} % Pass through any extra options specified to the base class
\ProcessOptions\relax % Process class options

\LoadClass{extarticle} % Load the base class

%----------------------------------------------------------------------------------------
%	REQUIRED PACKAGES AND MISC CONFIGURATIONS
%----------------------------------------------------------------------------------------

\usepackage{fp} % Required for invoice calculations

\usepackage[ % Required for automatically formatting numbers with \num{}, such as adding commas for numbers like: 1200 -> 1,200
	detect-all, text-rm, % Detect the weight, family and shape of the current font and match it when outputting a number with \num{}
]{siunitx}

\usepackage{setspace} % Required to enable changing line spacing

\usepackage{etoolbox} % Required for conditional logic and easily changing commands

\usepackage[hidelinks]{hyperref} % For clickable links (e.g. emails and URLs)

\pagestyle{empty} % Suppress all headers and footers

\newlength{\negativesign}
\settowidth{\negativesign}{--} % Calculate and save the width of the negative sign for subtraction

\setlength\parindent{0pt} % Stop paragraph indentation

\usepackage[document]{ragged2e} % Left align all text in the document (i.e. have a ragged right margin)

%----------------------------------------------------------------------------------------
%	MARGINS
%----------------------------------------------------------------------------------------

\usepackage[
	top=2.5cm, % Top margin
	bottom=2.25cm, % Bottom margin
	left=2cm, % Left margin
	right=2cm, % Right margin
	%showframe % Uncomment to show frames around the margins for debugging purposes
]{geometry}

%----------------------------------------------------------------------------------------
%	FONTS
%----------------------------------------------------------------------------------------

\usepackage[utf8]{inputenc} % Required for inputting international characters
\usepackage[T1]{fontenc} % Output font encoding for international characters

\usepackage[default]{lato} % Use the Lato sans serif font

\usepackage{textcomp} % Required for currency symbols
\usepackage{tfrupee} % Required for the rupee symbol

%----------------------------------------------------------------------------------------
%	TABLES
%----------------------------------------------------------------------------------------

\usepackage{longtable} % Required for tables that can span multiple pages
\setlength{\LTpre}{0pt} % Whitespace above longtables
\setlength{\LTpost}{0pt} % Whitespace below longtables
\setlength{\LTleft}{0pt} % Whitespace to the left of longtables

%----------------------------------------

\usepackage{array} % Required for manipulating table columns

\renewcommand{\arraystretch}{1.7} % Increase the space between table rows

\newcolumntype{R}[1]{>{\raggedleft\arraybackslash}p{#1}} % Define a new right-aligned column type
\newcolumntype{L}[1]{>{\raggedright\arraybackslash}p{#1}} % Define a new left-aligned (no justification) column type
\newcolumntype{C}[1]{>{\centering\arraybackslash}p{#1}} % Define a new centered column type

%----------------------------------------

\setlength{\tabcolsep}{4pt} % Decrease default padding between columns (6pt is the default)

%----------------------------------------------------------------------------------------
%	DETERMINE CURRENCY SYMBOL
%----------------------------------------------------------------------------------------

\newcommand{\currencysymbol}{\$} % The default currency symbol is a dollar sign

\newcommand{\determinecurrencysymbol}[1]{ % Uses the user-submitted currency to determine the symbol to use
	\expandafter\ifstrequal\expandafter{#1}{GBP}{\renewcommand{\currencysymbol}{\textsterling}}{} % GBP (English pounds)
	
	\ifboolexpr{ test {\expandafter\ifstrequal\expandafter{#1}{JPY}} or test {\expandafter\ifstrequal\expandafter{#1}{CNY}} }{\renewcommand{\currencysymbol}{\textyen}}{} % JPY or CNY (yen)
	
	\expandafter\ifstrequal\expandafter{#1}{EUR}{\renewcommand{\currencysymbol}{\texteuro}}{} % EUR (European euro)
	
	\expandafter\ifstrequal\expandafter{#1}{BRL}{\renewcommand{\currencysymbol}{R\$}}{} % BRL (Brazilian real)
	
	\expandafter\ifstrequal\expandafter{#1}{INR}{\renewcommand{\currencysymbol}{\rupee}}{} % INR (Indian rupee)
}

%----------------------------------------------------------------------------------------
%	DUE DATE CALCULATION
%----------------------------------------------------------------------------------------

\usepackage{datenumber} % Required for date calculations

\newcommand{\duedatedays}[1]{% Command to calculate and output the due date from the number of days from today
	\addtocounter{datenumber}{#1}% Number of days from today
	\setdatebynumber{\thedatenumber}% Calculate into the future using the number of days from today
	\datedate % Output the calculated date
}

%----------------------------------------------------------------------------------------
%	INVOICE ENTRIES
%----------------------------------------------------------------------------------------

% Define global variables that get updated for each new invoice item for various invoice totals
\gdef\currentsubtotalglobal{0} % Variable to store the current invoice item's subtotal
\gdef\totalbeforetaxglobal{0} % Cumulative variable storing the total before tax
\gdef\totaltaxglobal{0} % Cumulative variable storing the total tax
\gdef\totalaftertaxglobal{0} % Cumulative variable storing the total after tax

%----------------------------------------

% Command to calculate the subtotal for an invoice item and add it to the running totals for the whole invoice
\newcommand{\calculatesubtotalandupdatetotals}[2]{% Takes 2 arguments: 1) quantity 2) unit price
	\FPmul{\subtotal}{#1}{#2}% Calculate the subtotal by multiplying the quantity by the unit price
	\FPround{\subtotal}{\subtotal}{\roundcurrencytodecimals}% Round the subtotal to the specified number of decimal places for display and further calculations
	\global\let\currentsubtotalglobal\subtotal% Set the global current subtotal variable for further use and output to the invoice
	%----------------------------------------
	\FPeval{\beforetax}{round(\totalbeforetaxglobal + \currentsubtotalglobal, \roundcurrencytodecimals)}% Calculate the cumulative before tax total for the whole invoice by adding the current line's subtotal to the previous cumulative before tax total and round to the specified number of decimal places for display
	\global\let\totalbeforetaxglobal\beforetax% Update the global cumulative before tax total variable for further use and output to the invoice
	%----------------------------------------
	\ifdefempty{\taxrate}{}{% If a tax rate was set
		\FPeval{\tax}{round(\totalbeforetaxglobal * (\taxrate / 100), \roundcurrencytodecimals)}% Calculate the cumulative total tax for the whole invoice using the total cumulative before tax total and the tax rate, rounded to the specified number of decimal places for display
		\global\let\totaltaxglobal\tax% Update the global tax total variable for further use and output to the invoice
	}%
	%----------------------------------------
	\FPeval{\aftertax}{round(\totalbeforetaxglobal + \totaltaxglobal, \roundcurrencytodecimals)}% Calculate the cumulative after tax total for the whole invoice by adding the before tax total to the total tax and rounding to the specified number of decimal places for display
	\global\let\totalaftertaxglobal\aftertax% Update the global after tax total variable for further use and output to the invoice
}

%----------------------------------------

% Command to correctly output a currency number to the invoice, taking into account negatives and currency symbol positioning
\newcommand{\outputcurrency}[1]{%
	\FPifneg{#1}% If the number is negative
		\FPmul{\positivenumber}{#1}{-1}% Make the negative number positive
		\hspace{-\negativesign}--\currencysymbol\FPeval{\roundedpositivenumber}{round(\positivenumber, \roundcurrencytodecimals)}\num{\roundedpositivenumber}\currencysuffix% Output the negative sign before the currency symbol (pulled left in case of left aligned table columns), otherwise it would be e.g. $-55.00, and round to the specified number of decimal places
	\else% If the number is positive
		\currencysymbol\FPeval{\roundednumber}{round(#1, \roundcurrencytodecimals)}\num{\roundednumber}\currencysuffix% Round to the specified number of decimal places
	\fi%
}

%----------------------------------------

\newcommand{\invoiceitem}[4]{ % Arguments: 1) description 2) quantity 3) unit price 4) note
	\calculatesubtotalandupdatetotals{#2}{#3}% Calculate the subtotal for the current item and update totals for the whole invoice
	#1 & % Output the item description
	\FPeval{\roundedquantity}{round(#2, \roundquantitytodecimals)}\num{\roundedquantity} & % Output the quantity to a table cell, rounded to the specified number of decimal places
	\outputcurrency{#3} & % Output the unit price to a table cell
	\outputcurrency{\currentsubtotalglobal} & % Output the subtotal to a table cell
	{\small #4}\\ % Output the item note to a table cell
}

%----------------------------------------------------------------------------------------
%	INVOICE TABLE ENVIRONMENT
%----------------------------------------------------------------------------------------

\newenvironment{invoicetable}{
	\vspace{0.02\textheight} % Fixed vertical whitespace in case the table uses up all the stretch space
	
	\begin{longtable}[H]{@{} L{0.38\textwidth} R{0.15\textwidth} R{0.15\textwidth} R{0.15\textwidth} @{\hspace{16pt}} L{0.15\textwidth} @{}} % Define invoice table column widths and alignments (L, R or C for left, right or center alignment)
		\textbf{DESCRIPTION} & \textbf{QUANTITY} & \textbf{UNIT PRICE} & \textbf{SUBTOTAL} & \\ % Header row
}{
		\\ % Extra line before the summary numbers
		\ifdefempty{\taxrate}{}{& & \textbf{BEFORE TAX} & \outputcurrency{\totalbeforetaxglobal}\\} % Don't output the before tax line if no tax has been set
		\ifdefempty{\taxrate}{}{& & \textbf{TAX (\taxrate\%)} & \outputcurrency{\totaltaxglobal}\\} % Don't output the total tax line if no tax has been set
		& & \textbf{TOTAL\ifdefempty{\currencycode}{}{ (\currencycode)}} & \outputcurrency{\totalaftertaxglobal}\\ % Don't output the currency code if it has not been set
	\end{longtable}
	
	\vspace{0.02\textheight} % Fixed vertical whitespace in case the table uses up all the stretch space
}

%----------------------------------------------------------------------------------------
%	INVOICE HEADER DEFINITION
%----------------------------------------------------------------------------------------

\newcommand{\outputheader}[2]{
	{\Huge\MakeUppercase{#1}} % Document title
	
	\vspace{0.015\textheight} % Vertical whitespace
	
	\expandafter\ifstrequal\expandafter{#2}{}{}{{\large #2}} % Invoice date (won't be output if empty)
	
	\vspace{0.1\textheight} % Vertical whitespace
}

%----------------------------------------------------------------------------------------
%	INVOICE NUMBER
%----------------------------------------------------------------------------------------

\newcommand{\outputinvoicenum}{
	\ifdefempty{\invoicenumber}{}{ % If the invoice number has been set
		{\Huge\#\invoicenumber} % Output the invoice number
		\smallskip % Vertical whitespace
	}
}

%----------------------------------------------------------------------------------------
%	INVOICE CONDITIONS
%----------------------------------------------------------------------------------------

\newcommand{\invoiceconditions}[1]{
	\expandafter\ifstrequal\expandafter{#1}{}{}{
		{\justifying #1 \par} % Justify the text
	}
	\vspace{0.05\textheight} % Vertical whitespace
}

%----------------------------------------------------------------------------------------
%	CUSTOM COMMANDS
%----------------------------------------------------------------------------------------

\newcommand{\taxrate}[1]{\renewcommand{\taxrate}{#1}}
\newcommand{\currencycode}[1]{\renewcommand{\currencycode}{#1}}
\newcommand{\invoicenumber}[1]{\renewcommand{\invoicenumber}{#1}}
\newcommand{\currencysuffix}[1]{\renewcommand{\currencysuffix}{#1}}
\newcommand{\roundquantitytodecimals}[1]{\renewcommand{\roundquantitytodecimals}{#1}}
\newcommand{\roundcurrencytodecimals}[1]{\renewcommand{\roundcurrencytodecimals}{#1}}
